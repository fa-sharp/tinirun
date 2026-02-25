use aide::OperationOutput;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tinirun_models::CodeRunnerError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),
    #[error("Docker client error: {0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("Execution failed: {0}")]
    ExecutionFailed(CodeRunnerError),
    #[error("Server error: {0}")]
    Server(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Serialization(err) => {
                tracing::warn!("Serialization error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            AppError::Redis(err) => {
                tracing::error!("Redis error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::ExecutionFailed(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Code execution failed: {err}"),
            )
                .into_response(),
            AppError::Server(err) => {
                tracing::warn!("Server error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            AppError::Docker(err) => {
                tracing::error!("Docker client error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

impl OperationOutput for AppError {
    type Inner = String;
}
