use reqwest::StatusCode;
use thiserror::Error;

use crate::oauth::error::OauthError;

#[derive(Debug, Error)]
pub enum FotolifeError {
  #[error("failed to open requested resource: {resource:?}")]
  ResourceNotFound { resource: String },

  #[error("request failed")]
  RequestFailure(#[from] OauthError),

  #[error("failed to upload image (status={status:?})")]
  UploadFailure { status: StatusCode },

  #[error("request failed")]
  HttpFailure(#[from] reqwest::Error),
}
