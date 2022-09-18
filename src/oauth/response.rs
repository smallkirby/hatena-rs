use std::collections::HashMap;

use crate::oauth::error::*;

use percent_encoding::percent_decode;

#[derive(Debug)]
pub struct OauthTokenResponse {
  pub oauth_token: String,
  pub oauth_token_secret: String,
}

impl OauthTokenResponse {
  pub fn from(response: &str) -> Result<Self, OauthError> {
    let mut map: HashMap<&str, String> = HashMap::new();
    for pair in response.split('&') {
      let mut parts = pair.split('=');
      let key = parts.next();
      let value = parts.next();
      if key.is_none() || value.is_none() {
        return Err(OauthError::InvalidResponse {
          response: response.to_string(),
        });
      }
      map.insert(
        key.unwrap(),
        percent_decode(value.unwrap().as_bytes())
          .decode_utf8_lossy()
          .to_string(),
      );
    }

    let oauth_token = map.get("oauth_token").ok_or(OauthError::InvalidResponse {
      response: response.to_string(),
    })?;
    let oauth_token_secret = map
      .get("oauth_token_secret")
      .ok_or(OauthError::InvalidResponse {
        response: response.to_string(),
      })?;

    Ok(Self {
      oauth_token: oauth_token.to_string(),
      oauth_token_secret: oauth_token_secret.to_string(),
    })
  }
}

#[derive(Debug)]
pub struct AccessTokenResponse {
  pub oauth_token: String,
  pub oauth_token_secret: String,
  pub url_name: String,
  pub display_name: String,
}

impl AccessTokenResponse {
  pub fn from(response: &str) -> Result<Self, OauthError> {
    let mut map: HashMap<&str, String> = HashMap::new();
    for pair in response.split('&') {
      let mut parts = pair.split('=');
      let key = parts.next();
      let value = parts.next();
      if key.is_none() || value.is_none() {
        return Err(OauthError::InvalidResponse {
          response: response.to_string(),
        });
      }
      map.insert(
        key.unwrap(),
        percent_decode(value.unwrap().as_bytes())
          .decode_utf8_lossy()
          .to_string(),
      );
    }

    let oauth_token = map
      .get("oauth_token")
      .ok_or(OauthError::InvalidResponse {
        response: response.to_string(),
      })?
      .to_string();
    let oauth_token_secret = map
      .get("oauth_token_secret")
      .ok_or(OauthError::InvalidResponse {
        response: response.to_string(),
      })?
      .to_string();
    let url_name = map
      .get("url_name")
      .ok_or(OauthError::InvalidResponse {
        response: response.to_string(),
      })?
      .to_string();
    let display_name = map
      .get("display_name")
      .ok_or(OauthError::InvalidResponse {
        response: response.to_string(),
      })?
      .to_string();

    Ok(Self {
      oauth_token_secret,
      oauth_token,
      url_name,
      display_name,
    })
  }
}
