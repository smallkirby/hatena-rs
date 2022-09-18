use std::collections::HashMap;
use std::env;
use std::io::Write;

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::header::AUTHORIZATION;
use ring::hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use webbrowser;

use crate::oauth::consts::*;
use crate::oauth::error::*;
use crate::oauth::response::*;
use crate::oauth::util::*;

struct RequestToken {
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

  pub fn to_header_string(&self, url: &str, params: Option<&HashMap<&str, &str>>) -> String {
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

    let signature = self.get_signature(url, &headers);
    headers.insert("oauth_signature", &signature);

    let mut header_strs = headers
      .iter()
      .filter(|&(k, _)| k.starts_with("oauth_"))
      .map(|(k, v)| format!("{}=\"{}\"", k, &encode(v)))
      .collect::<Vec<String>>();
    header_strs.sort();

    format!("OAuth {}", header_strs.join(", "),)
  }

  fn get_signature(&self, url: &str, params: &HashMap<&str, &str>) -> String {
    let key = format!(
      "{}&{}",
      encode(&self.consumer_secret),
      encode(&self.oauth_token_secret.clone().unwrap_or("".into()))
    );

    let mut sorted_params: Vec<_> = params
      .iter()
      .filter(|&(k, _)| k.starts_with("oauth_"))
      .map(|(k, v)| format!("{}={}", encode(k), encode(v),))
      .collect::<Vec<String>>();
    sorted_params.sort();

    let base_string = format!(
      "{}&{}&{}",
      encode("POST"),
      encode(url),
      encode(&sorted_params.join("&")),
    );

    let sign_key = hmac::Key::new(HMAC_SHA1_FOR_LEGACY_USE_ONLY, key.as_ref());
    let signature = hmac::sign(&sign_key, base_string.as_bytes());

    base64::encode(&signature.as_ref())
  }
}

pub fn get_request_token() -> Result<OauthTokenResponse, OauthError> {
  let consumer_key = if let Ok(val) = env::var(ENV_CONSUMER_KEY) {
    val
  } else {
    return Err(OauthError::InsufficientSecret);
  };
  let consumer_secret = if let Ok(val) = env::var(ENV_CONSUMER_SECRET) {
    val
  } else {
    return Err(OauthError::InsufficientSecret);
  };
  let params: HashMap<&str, &str> = vec![("oauth_callback", "oob")].into_iter().collect();
  let req_token = RequestToken::new(&consumer_key, &consumer_secret, None, None);
  let scopes = vec![
    "read_public",
    "write_public",
    "read_private",
    "write_private",
  ];

  let client = reqwest::blocking::Client::new();
  let res = client
    .post("https://www.hatena.com/oauth/initiate")
    .header(
      AUTHORIZATION,
      req_token.to_header_string("https://www.hatena.com/oauth/initiate", Some(&params)),
    )
    .body(format!("scope={}", encode(&scopes.join("&")),))
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

pub fn grant_permission_browser(token: &OauthTokenResponse) -> Result<String, ()> {
  let result = webbrowser::open(&format!(
    "https://www.hatena.ne.jp/oauth/authorize?oauth_token={}",
    token.oauth_token
  ));
  if result.is_err() {
    return Err(());
  }

  let mut oauth_verifier = String::new();
  print!(
    "Input token printed on the browser (or, 'set {}=<token>' and Enter): ",
    ENV_OAUTH_VERIFIER
  );
  std::io::stdout().flush().unwrap();
  std::io::stdin().read_line(&mut oauth_verifier).unwrap();

  if oauth_verifier.trim().is_empty() {
    if let Ok(val) = env::var(ENV_OAUTH_VERIFIER) {
      oauth_verifier = val;
    } else {
      return Err(());
    }
  } else {
    oauth_verifier = oauth_verifier.trim().to_string();
  }

  Ok(oauth_verifier)
}

pub fn get_access_token(
  token: &OauthTokenResponse,
  oauth_verifier: &str,
) -> Result<AccessTokenResponse, OauthError> {
  let consumer_key = if let Ok(val) = env::var(ENV_CONSUMER_KEY) {
    val
  } else {
    return Err(OauthError::InsufficientSecret);
  };
  let consumer_secret = if let Ok(val) = env::var(ENV_CONSUMER_SECRET) {
    val
  } else {
    return Err(OauthError::InsufficientSecret);
  };
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
    .post("https://www.hatena.com/oauth/token")
    .header(
      AUTHORIZATION,
      req_token.to_header_string("https://www.hatena.com/oauth/token", Some(&params)),
    )
    .send();

  if let Ok(res) = res {
    if res.status() == 200 {
      let text = res.text()?;
      match AccessTokenResponse::from(&text) {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_request_token() {
    let token = get_request_token().unwrap();
    println!("{:?}", token);
  }

  #[test]
  fn test_grant_permission_browser() {
    let token = get_request_token().unwrap();
    let oauth_verifier = grant_permission_browser(&token).unwrap();
    println!("{:?}", oauth_verifier);
  }

  #[test]
  fn test_get_access_token() {
    let token = get_request_token().unwrap();
    let oauth_verifier = grant_permission_browser(&token).unwrap();
    let access_token = get_access_token(&token, &oauth_verifier).unwrap();
    println!("{:?}", access_token);
  }
}
