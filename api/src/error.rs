use std::fmt::format;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;


#[derive(thiserror::Error, Debug)]
pub enum CodeLlmTestApiError {
    #[error("Resource Not Found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Worker(#[from] worker::Error)
}

impl IntoResponse for CodeLlmTestApiError {
    fn into_response(self) -> Response {
        match self {
            CodeLlmTestApiError::NotFound(name) => {
                let err_msg = format!("Resource Not Found: {}", name);
                error!("{}", err_msg);
                (StatusCode::NOT_FOUND, err_msg)
            }
            CodeLlmTestApiError::Worker(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("worker error: {}", e),
                )
            }
        }
        .into_response()
    }
}
