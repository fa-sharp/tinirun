//! API input validators

use axum::response::IntoResponse;
use serde::Serialize;

mod json;
mod stream_type;

pub use json::AppValidJson;
pub use stream_type::StreamType;

#[derive(Debug, Serialize)]
pub struct InputValidationError {
    message: String,
}

impl InputValidationError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl IntoResponse for InputValidationError {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
