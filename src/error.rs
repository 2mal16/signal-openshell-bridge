use crate::config::PhoneNumber;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("account not allowed: {0:?}")]
    NumberNotAllowed(PhoneNumber),

    #[error("no data message found")]
    NoMessage(),

    #[error("empty data message found")]
    EmptyMessage(),

    #[error("account not allowed: {0:?}")]
    AccountNotAllowed(String),

    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}
