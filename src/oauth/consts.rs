use percent_encoding::AsciiSet;

pub static STRICT_ENCODE_SET: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
  .remove(b'*')
  .remove(b'-')
  .remove(b'.')
  .remove(b'_');

pub const ENV_CONSUMER_KEY: &str = "HATENA_CONSUMER_KEY";
pub const ENV_CONSUMER_SECRET: &str = "HATENA_CONSUMER_SECRET";
pub const ENV_OAUTH_VERIFIER: &str = "HATENA_OAUTH_VERIFIER";
