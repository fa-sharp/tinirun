use aide::OperationInput;
use axum::extract::FromRequestParts;

use crate::state::AppState;

/// Extractor to determine the stream response type from the `Accept` header.
/// SSE used by default if no valid header/type is provided.
#[derive(Default)]
pub enum StreamType {
    /// Server-Sent Events
    #[default]
    Sse,
    /// JSON Lines / Newline-delimited JSON
    Jsonl,
}

impl FromRequestParts<AppState> for StreamType {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts
            .headers
            .get(axum::http::header::ACCEPT)
            .map(|value| value.as_bytes())
            .and_then(|accept_header| match accept_header {
                b"text/event-stream" => Some(StreamType::Sse),
                b"application/json"
                | b"application/jsonl"
                | b"application/json-lines"
                | b"application/x-ndjson" => Some(StreamType::Jsonl),
                _ => Some(StreamType::default()),
            })
            .unwrap_or_default())
    }
}

impl OperationInput for StreamType {}
