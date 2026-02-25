use std::time::Duration;

use futures::Stream;
use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue};
use reqwest_streams::{JsonStreamResponse, error::StreamBodyError};
use tinirun_models::{CodeRunnerChunk, CodeRunnerInput};
use validator::Validate;

/// # Tinirun client
/// A Rust client for the Tinirun API that supports streaming code execution logs and results.
pub struct TinirunClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TinirunError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Input validation error(s): {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("API error: {status} {message:?}")]
    Api {
        status: u16,
        message: Option<String>,
    },
}

impl TinirunClient {
    pub fn new(base_url: impl Into<String>, api_key: impl AsRef<str>) -> Self {
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::limited(2))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("tinirun-client/{}", env!("CARGO_PKG_VERSION")))
            .default_headers(HeaderMap::from_iter([(
                HeaderName::from_static("X-Runner-Api-Key"),
                HeaderValue::from_str(api_key.as_ref()).expect("Invalid API key value"),
            )]))
            .build()
            .expect("Failed to build client");

        TinirunClient {
            client,
            base_url: base_url.into(),
        }
    }

    pub async fn run_code(
        &self,
        input: &CodeRunnerInput,
    ) -> Result<impl Stream<Item = Result<CodeRunnerChunk, StreamBodyError>>, TinirunError> {
        if let Err(e) = input.validate() {
            return Err(TinirunError::Validation(e));
        }

        let response = self
            .client
            .post(format!("{}/code/run", self.base_url))
            .header(ACCEPT, "application/jsonl")
            .json(input)
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = extract_error_message(response).await;
            return Err(TinirunError::Api { status, message });
        }

        Ok(response.json_nl_stream::<CodeRunnerChunk>(64 * 1024))
    }
}

async fn extract_error_message(response: reqwest::Response) -> Option<String> {
    if let Ok(response_text) = response.text().await {
        if let Ok(value) = serde_json::to_value(&response_text) {
            if let Some(message) = value["message"].as_str() {
                return Some(message.to_owned());
            }
        } else {
            return Some(response_text);
        }
    }
    None
}
