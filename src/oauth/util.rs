use crate::oauth::consts::STRICT_ENCODE_SET;

use percent_encoding::percent_encode;

pub fn encode(s: &str) -> String {
  percent_encode(s.as_bytes(), STRICT_ENCODE_SET).collect()
}
