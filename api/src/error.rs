use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub(crate) type Result<T> = core::result::Result<T, ApiError>;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Resource Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate Limited: {0}")]
    RateLimited(String),

    #[error("OAuth Error: {0}")]
    OAuth(String),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error(transparent)]
    Worker(#[from] worker::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl ApiError {
    pub(crate) fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(msg.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::RateLimited(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            ApiError::OAuth(msg) => (StatusCode::BAD_GATEWAY, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Worker(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("worker error: {}", e)),
            ApiError::SerdeJson(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("json error: {}", e)),
        }
        .into_response()
    }
}
