//! Shared error type used across the backend.

use serde::{Serialize, Serializer};
use thiserror::Error;

/// Every error that the backend can surface to the frontend.
/// It is serialized as a plain string so the JS side can do `throw new Error(msg)`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("database pool error: {0}")]
    DbPool(#[from] r2d2::Error),

    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("cloudflare challenge required")]
    Cloudflare,

    #[error("authentication required (401/403)")]
    Unauthorized,

    #[error("invalid response from server")]
    InvalidResponse,

    #[error("gallery not found")]
    NotFound,

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
