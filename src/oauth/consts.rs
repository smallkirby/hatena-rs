use std::fmt;

use percent_encoding::AsciiSet;

pub static STRICT_ENCODE_SET: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
  .remove(b'*')
  .remove(b'-')
  .remove(b'.')
  .remove(b'_');

pub const ENV_CONSUMER_KEY: &str = "HATENA_CONSUMER_KEY";
pub const ENV_CONSUMER_SECRET: &str = "HATENA_CONSUMER_SECRET";
pub const ENV_OAUTH_VERIFIER: &str = "HATENA_OAUTH_VERIFIER";
pub const ENV_OAUTH_ACCESS_TOKEN: &str = "HATENA_OAUTH_ACCESS_TOKEN";
pub const ENV_OAUTH_ACCESS_SECRET: &str = "HATENA_OAUTH_ACCESS_SECRET";
pub const ENV_OAUTH_URL_NAME: &str = "HATENA_OAUTH_URL_NAME";

pub const OAUTH_URL_REQUEST_TOKEN: &str = "https://www.hatena.com/oauth/initiate";
pub const OAUTH_URL_GRANT_PERMISSION: &str = "https://www.hatena.com/oauth/authorize";
pub const OAUTH_URL_ACCESS_TOKEN: &str = "https://www.hatena.com/oauth/token";

pub enum OauthScope {
  ReadPublic,
  ReadPrivate,
  WritePublic,
  WritePrivate,
}

impl fmt::Display for OauthScope {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      OauthScope::ReadPublic => write!(f, "read_public"),
      OauthScope::ReadPrivate => write!(f, "read_private"),
      OauthScope::WritePublic => write!(f, "write_public"),
      OauthScope::WritePrivate => write!(f, "write_private"),
    }
  }
}
