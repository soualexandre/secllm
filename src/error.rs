//! Unified error type and conversions for SecLLM.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("vault error: {0}")]
    Vault(String),

    #[error("privacy/scan error: {0}")]
    Privacy(String),

    #[error("proxy/forward error: {0}")]
    Proxy(String),

    #[error("logging error: {0}")]
    Logging(String),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Vault(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Privacy(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Proxy(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Logging(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        let body = ErrorBody { error: message };
        (status, Json(body)).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

pub type Result<T> = std::result::Result<T, AppError>;
