use thiserror::Error;

#[derive(Debug, Error)]
pub enum OauthError {
  #[error("request failed")]
  RequestFailure(#[from] reqwest::Error),

  #[error("invalid request ({problem:?})")]
  InvalidRequest { problem: String },

  #[error("invalid response format: {response:?}")]
  InvalidResponse { response: String },

  #[error("HATENA_CONSUMER_KEY or HATENA_CONSUMER_SECRET is not set")]
  InsufficientSecret,

  #[error("permission denied by yourself")]
  PermissionDeniedUser,
}
