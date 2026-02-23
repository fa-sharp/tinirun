use axum::extract::State;
use futures::Stream;

use crate::{
    input::{AppValidJson, StreamType},
    responses::StreamResponse,
    runner::{CodeRunnerChunk, CodeRunnerInput},
    state::AppState,
};

pub fn route() -> aide::axum::routing::ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("run_code")
            .summary("Run code")
            .description("Run a one-off script with the given parameters and stream the output")
    })
}

async fn handler(
    State(state): State<AppState>,
    stream_type: StreamType,
    AppValidJson(input): AppValidJson<CodeRunnerInput>,
) -> Result<StreamResponse<impl Stream<Item = CodeRunnerChunk>, CodeRunnerChunk>, String> {
    let stream = state
        .runner
        .execute(input)
        .await
        .map_err(|err| err.to_string())?;

    Ok(StreamResponse::new(stream, stream_type))
}
