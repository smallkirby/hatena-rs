mod consts;
mod error;
mod oauth;
mod response;
mod util;

use crate::oauth::consts::*;
use crate::oauth::error::*;
use crate::oauth::oauth::*;
use crate::oauth::response::*;
use std::env;

/// OAuth client instance
pub struct HatenaOauth {
  /// Consumer key
  consumer_key: String,
  /// Consumer secret
  consumer_secret: String,
  /// Scopes granted for the access token
  scopes: Vec<OauthScope>,
  /// Cache of request token response
  request_token: Option<OauthTokenResponse>,
  /// Cache of access token response
  access_token: Option<AccessTokenResponse>,
  /// Cache of oauth verifier
  verifier: Option<String>,
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
  pub fn new(scopes: Vec<OauthScope>) -> Result<Self, OauthError> {
    let consumer_key = env::var(ENV_CONSUMER_KEY).map_err(|_| OauthError::InsufficientSecret)?;
    let consumer_secret =
      env::var(ENV_CONSUMER_SECRET).map_err(|_| OauthError::InsufficientSecret)?;
    let access_token = get_access_token_from_env();

    Ok(Self {
      consumer_key,
      consumer_secret,
      scopes,
      request_token: None,
      access_token,
      verifier: None,
    })
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
      &self.consumer_key,
      &self.consumer_secret,
    )?);

    Ok(self.access_token.as_ref().unwrap().clone())
  }

  fn get_request_token(&mut self) -> Result<(), OauthError> {
    self.request_token = Some(get_request_token(
      &self.scopes,
      &self.consumer_key,
      &self.consumer_secret,
    )?);

    Ok(())
  }

  fn get_verifier(&mut self) -> Result<(), OauthError> {
    self.verifier = Some(grant_permission_browser(
      &self.request_token.as_ref().unwrap(),
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
    let mut oauth = HatenaOauth::new(vec![OauthScope::ReadPublic]).unwrap();
    let token = oauth.get_access_token(false).unwrap();
    println!("{:?}", token);
  }
}
