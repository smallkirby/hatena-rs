use std::collections::HashMap;
use std::env;
use std::io::Write;

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use ring::hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use webbrowser;

use crate::oauth::consts::*;
use crate::oauth::error::*;
use crate::oauth::token::*;
use crate::oauth::util::*;

pub struct RequestToken {
  consumer_key: String,
  consumer_secret: String,
  oauth_token: Option<String>,
  oauth_token_secret: Option<String>,
}

impl RequestToken {
  pub fn new(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_token: Option<&str>,
    oauth_token_secret: Option<&str>,
  ) -> Self {
    RequestToken {
      consumer_key: consumer_key.to_string(),
      consumer_secret: consumer_secret.to_string(),
      oauth_token: oauth_token.map(|s| s.to_string()),
      oauth_token_secret: oauth_token_secret.map(|s| s.to_string()),
    }
  }

  pub fn to_header_string(
    &self,
    url: &str,
    method: &str,
    params: Option<&HashMap<&str, &str>>,
    body: Option<&str>,
  ) -> String {
    let mut headers: HashMap<&str, &str> = match params {
      Some(map) => map.clone(),
      None => HashMap::new(),
    };
    let timestamp = &format!("{}", &Utc::now().timestamp());
    let nonce: String = thread_rng()
      .sample_iter(&Alphanumeric)
      .take(32)
      .map(char::from)
      .collect();

    headers.insert("oauth_consumer_key", &self.consumer_key);
    headers.insert("oauth_nonce", &nonce);
    headers.insert("oauth_signature_method", "HMAC-SHA1");
    headers.insert("oauth_timestamp", timestamp);
    headers.insert("oauth_version", "1.0");
    if let Some(oauth_token) = &self.oauth_token {
      headers.insert("oauth_token", oauth_token);
    }

    let mut params_for_signature = headers.clone();
    if body.is_some() {
      let body = body.unwrap();
      for pair in body.split('&') {
        let mut parts = pair.split('=');
        let key = parts.next();
        let value = parts.next();
        if key.is_none() || value.is_none() {
          continue;
        }
        params_for_signature.insert(key.unwrap(), value.unwrap());
      }
    }
    let signature = self.get_signature(url, method, &params_for_signature);
    headers.insert("oauth_signature", &signature);

    let mut header_strs = headers
      .iter()
      .filter(|&(k, _)| k.starts_with("oauth_"))
      .map(|(k, v)| format!("{}=\"{}\"", k, &encode(v)))
      .collect::<Vec<String>>();
    header_strs.sort();

    format!("OAuth {}", header_strs.join(", "),)
  }

  fn get_signature(&self, url: &str, method: &str, params: &HashMap<&str, &str>) -> String {
    let key = format!(
      "{}&{}",
      encode(&self.consumer_secret),
      encode(&self.oauth_token_secret.clone().unwrap_or("".into()))
    );

    let mut sorted_params: Vec<_> = params
      .iter()
      .map(|(k, v)| format!("{}={}", encode(k), encode(v),))
      .collect::<Vec<String>>();
    sorted_params.sort();

    let base_string = format!(
      "{}&{}&{}",
      encode(method),
      encode(url),
      encode(&sorted_params.join("&")),
    );

    let sign_key = hmac::Key::new(HMAC_SHA1_FOR_LEGACY_USE_ONLY, key.as_ref());
    let signature = hmac::sign(&sign_key, base_string.as_bytes());

    base64::encode(&signature.as_ref())
  }
}

/// Get a request token with specified scopes;
///
/// # Arguments
///
/// * `scopes` - A list of scopes to request
/// * `consumer_key` - A consumer key for Hatena OAuth
/// * `consumer_secret` - A consumer secret for Hatena OAuth
pub fn get_request_token(
  scopes: &Vec<OauthScope>,
  consumer_key: &str,
  consumer_secret: &str,
) -> Result<OauthTokenResponse, OauthError> {
  let params: HashMap<&str, &str> = vec![("oauth_callback", "oob")].into_iter().collect();
  let req_token = RequestToken::new(&consumer_key, &consumer_secret, None, None);
  let scopes_str = scopes
    .iter()
    .map(|s| format!("{}", s))
    .collect::<Vec<String>>()
    .join(",");

  let client = reqwest::blocking::Client::new();
  let res = client
    .post(OAUTH_URL_REQUEST_TOKEN)
    .header(
      AUTHORIZATION,
      req_token.to_header_string(
        OAUTH_URL_REQUEST_TOKEN,
        "POST",
        Some(&params),
        Some(&format!("scope={}", scopes_str)),
      ),
    )
    .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
    .body(format!("scope={}", encode(&scopes_str)))
    .send();

  if let Ok(res) = res {
    if res.status() == 200 {
      let text = res.text()?;
      match OauthTokenResponse::from(&text) {
        Ok(token) => Ok(token),
        Err(e) => Err(e),
      }
    } else {
      Err(OauthError::InvalidRequest {
        problem: res.text()?,
      })
    }
  } else {
    Err(OauthError::RequestFailure(res.unwrap_err()))
  }
}

/// Grant a permission from a user to get an access token.
///
/// This function opens a browser and waits for a user to grant a permission.
///
/// # Arguments
///
/// * `token` - A request token returned from request endpoint
pub fn grant_permission_browser(token: &OauthTokenResponse) -> Result<String, OauthError> {
  webbrowser::open(&format!(
    "{}?oauth_token={}",
    OAUTH_URL_GRANT_PERMISSION, token.oauth_token,
  ))
  .map_err(|_| OauthError::PermissionDeniedUser)?;

  let mut oauth_verifier = String::new();
  print!(
    "Input token printed on the browser (or, 'set {}=<token>' and Enter): ",
    ENV_OAUTH_VERIFIER
  );
  std::io::stdout().flush().unwrap();
  std::io::stdin().read_line(&mut oauth_verifier).unwrap();

  oauth_verifier = if oauth_verifier.trim().is_empty() {
    env::var(ENV_OAUTH_VERIFIER).map_err(|_| OauthError::PermissionDeniedUser)?
  } else {
    oauth_verifier.trim().to_string()
  };

  Ok(oauth_verifier)
}

/// Get an access token
///
/// # Arguments
///
/// * `token` - A request token returned from request endpoint
/// * `oauth_verifier` - OAuth verifier returned from authorization endpoint
/// * `consumer_key` - A consumer key for Hatena OAuth
/// * `consumer_secret` - A consumer secret for Hatena OAuth
pub fn get_access_token(
  token: &OauthTokenResponse,
  oauth_verifier: &str,
  consumer_key: &str,
  consumer_secret: &str,
) -> Result<AccessTokenResponse, OauthError> {
  let req_token = RequestToken::new(
    &consumer_key,
    &consumer_secret,
    Some(&token.oauth_token),
    Some(&token.oauth_token_secret),
  );
  let params: HashMap<&str, &str> = vec![("oauth_verifier", oauth_verifier)]
    .into_iter()
    .collect();

  let client = reqwest::blocking::Client::new();
  let res = client
    .post(OAUTH_URL_ACCESS_TOKEN)
    .header(
      AUTHORIZATION,
      req_token.to_header_string(OAUTH_URL_ACCESS_TOKEN, "POST", Some(&params), None),
    )
    .send();

  if let Ok(res) = res {
    if res.status() == 200 {
      let text = res.text()?;
      Ok(AccessTokenResponse::from(&text).map_err(|e| e)?)
    } else {
      Err(OauthError::InvalidRequest {
        problem: res.text()?,
      })
    }
  } else {
    Err(OauthError::RequestFailure(res.unwrap_err()))
  }
}
