pub mod consts;
pub mod error;
mod oauth;
pub mod token;
mod util;

use std::env;

use crate::oauth::consts::*;
use crate::oauth::error::*;
use crate::oauth::oauth::*;
use crate::oauth::token::*;

use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;

/// OAuth key info
#[derive(Debug, Clone)]
pub struct HatenaConsumerInfo {
  /// Consumer key
  consumer_key: String,
  /// Consumer secret
  consumer_secret: String,
}

impl HatenaConsumerInfo {
  pub fn new(consumer_key: &str, consumer_secret: &str) -> Result<Self, OauthError> {
    Ok(Self {
      consumer_key: consumer_key.to_string(),
      consumer_secret: consumer_secret.to_string(),
    })
  }

  pub fn from_env() -> Result<Self, OauthError> {
    let consumer_key =
      env::var("HATENA_CONSUMER_KEY").map_err(|_| OauthError::InsufficientSecret)?;
    let consumer_secret =
      env::var("HATENA_CONSUMER_SECRET").map_err(|_| OauthError::InsufficientSecret)?;

    Ok(Self {
      consumer_key,
      consumer_secret,
    })
  }
}

/// OAuth client instance
pub struct HatenaOauth {
  consumer_info: HatenaConsumerInfo,
  /// Scopes granted for the access token
  scopes: Vec<OauthScope>,
  /// Cache of request token response
  request_token: Option<OauthTokenResponse>,
  /// Cache of access token response
  access_token: Option<AccessTokenResponse>,
  /// Cache of oauth verifier
  verifier: Option<String>,
  /// Callback after redirecting user to grant permission
  grant_permission_callback: fn() -> Result<String, OauthError>,
}

impl HatenaOauth {
  /// Create a new OAuth client instance
  ///
  /// It gets consumer key and consumer secret from environment variables. If not exist, it returns `InsufficientSecret` error.
  ///
  /// It gets cached access token from environment variables if exist.
  ///
  /// # Arguments
  ///
  /// * `scopes` - Scopes to be requested for the access token
  /// * `grant_permission_callback` - Callback after redirecting user to grant permission, which prompts user to input a given token
  /// * `consumer_info` - A consumer info for Hatena OAuth
  pub fn new(
    scopes: Vec<OauthScope>,
    grant_permission_callback: Option<fn() -> Result<String, OauthError>>,
    consumer_info: HatenaConsumerInfo,
  ) -> Result<Self, OauthError> {
    let access_token = get_access_token_from_env();
    let callback = if let Some(callback) = grant_permission_callback {
      callback
    } else {
      || grant_permission_default_callback()
    };

    Ok(Self {
      consumer_info,
      scopes,
      request_token: None,
      access_token,
      verifier: None,
      grant_permission_callback: callback,
    })
  }

  /// Send GET request with OAuth Acess Token
  ///
  /// If access token is not cached, it first fetches access token.
  ///
  /// # Arguments
  ///
  /// * `url` - URL to send GET request
  /// * `force` - If true, it fetches access token even if it is cached
  pub fn get(&mut self, url: &str, force: bool) -> Result<Response, OauthError> {
    if force || self.access_token.is_none() {
      self.get_access_token(true)?;
    }

    let req_token = RequestToken::new(
      &self.consumer_info.consumer_key,
      &self.consumer_info.consumer_secret,
      Some(&self.access_token.as_ref().unwrap().oauth_token),
      Some(&self.access_token.as_ref().unwrap().oauth_token_secret),
    );
    let client = reqwest::blocking::Client::new();
    let response = client
      .get(url)
      .header(
        AUTHORIZATION,
        req_token.to_header_string(url, "GET", None, None),
      )
      .send()?;

    Ok(response)
  }

  /// Send POST request with OAuth Acess Token
  ///
  /// If access token is not cached, it first fetches access token.
  ///
  /// # Arguments
  ///
  /// * `url` - URL to send POST request
  /// * `body` - body of POST request to send
  /// * `force` - If true, it fetches access token even if it is cached
  /// * `timeout` - Timeout in seconds
  pub fn post(
    &mut self,
    url: &str,
    body: &str,
    force: bool,
    timeout: u64,
  ) -> Result<Response, OauthError> {
    if force || self.access_token.is_none() {
      self.get_access_token(true)?;
    }

    let req_token = RequestToken::new(
      &self.consumer_info.consumer_key,
      &self.consumer_info.consumer_secret,
      Some(&self.access_token.as_ref().unwrap().oauth_token),
      Some(&self.access_token.as_ref().unwrap().oauth_token_secret),
    );
    let client = reqwest::blocking::Client::new();
    let response = client
      .post(url)
      .timeout(std::time::Duration::from_secs(timeout))
      .header(
        AUTHORIZATION,
        req_token.to_header_string(url, "POST", None, Some("")),
      ) // XXX
      .body(body.to_string())
      .send()?;

    Ok(response)
  }

  /// Get an access token for pre-defined scopes.
  ///
  /// This function would open a browser and wait for a user to grant a permission.
  ///
  /// # Arguments
  ///
  /// * `force` - If true, this function would request a new access token even if the access token is already cached.
  pub fn get_access_token(&mut self, force: bool) -> Result<AccessTokenResponse, OauthError> {
    // Use cached access token if exists
    if !force && self.access_token.is_some() {
      return Ok(self.access_token.clone().unwrap());
    }

    // Use cached request token and verifier if exists
    if force || (self.request_token.is_none() || self.verifier.is_none()) {
      self.get_request_token()?;
      self.get_verifier()?;
    }

    self.access_token = Some(get_access_token(
      &self.request_token.as_ref().unwrap(),
      &self.verifier.as_ref().unwrap(),
      &self.consumer_info.consumer_key,
      &self.consumer_info.consumer_secret,
    )?);

    Ok(self.access_token.as_ref().unwrap().clone())
  }

  fn get_request_token(&mut self) -> Result<(), OauthError> {
    self.request_token = Some(get_request_token(
      &self.scopes,
      &self.consumer_info.consumer_key,
      &self.consumer_info.consumer_secret,
    )?);

    Ok(())
  }

  fn get_verifier(&mut self) -> Result<(), OauthError> {
    self.verifier = Some(grant_permission_browser(
      &self.request_token.as_ref().unwrap(),
      self.grant_permission_callback,
    )?);

    Ok(())
  }
}

fn get_access_token_from_env() -> Option<AccessTokenResponse> {
  let access_token = env::var(ENV_OAUTH_ACCESS_TOKEN).unwrap_or("".into());
  let access_secret = env::var(ENV_OAUTH_ACCESS_SECRET).unwrap_or("".into());
  let url_name = env::var(ENV_OAUTH_URL_NAME).unwrap_or("".into());

  if access_token.is_empty() || access_secret.is_empty() || url_name.is_empty() {
    None
  } else {
    Some(AccessTokenResponse {
      oauth_token: access_token,
      oauth_token_secret: access_secret,
      url_name: url_name.clone(),
      display_name: url_name,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_access_token() {
    let consumer_info = HatenaConsumerInfo::from_env().unwrap();
    let mut oauth = HatenaOauth::new(vec![OauthScope::ReadPublic], None, consumer_info).unwrap();
    let token = oauth.get_access_token(false).unwrap();
    println!("{:?}", token);
  }
}
