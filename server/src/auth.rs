use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use crate::state::AppState;

/// Extractor that checks for valid API key
pub struct ApiKey;

/// Header name for the API key
const API_KEY_HEADER: &str = "X-Runner-Api-Key";

pub enum ApiKeyError {
    Missing,
    Invalid,
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiKeyError::Missing => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiKeyError::Invalid => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        };
        (status, message).into_response()
    }
}

impl FromRequestParts<AppState> for ApiKey {
    type Rejection = ApiKeyError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let api_key = parts
            .headers
            .get(API_KEY_HEADER)
            .ok_or(ApiKeyError::Missing)?
            .to_str()
            .map_err(|_| ApiKeyError::Invalid)?;

        if api_key != state.config.api_key {
            return Err(ApiKeyError::Invalid);
        }

        Ok(ApiKey)
    }
}
