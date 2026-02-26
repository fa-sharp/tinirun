use axum::extract::State;
use futures::Stream;
use tinirun_models::{CodeRunnerChunk, CodeRunnerInput};

use crate::{
    api::ApiTag,
    errors::AppError,
    input::{AppJson, StreamType},
    responses::StreamResponse,
    state::AppState,
};

pub fn route() -> aide::axum::routing::ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("run_code")
            .tag(ApiTag::Run.into())
            .summary("Run code")
            .description("Run a one-off script with the given parameters and stream the output")
    })
}

async fn handler(
    State(state): State<AppState>,
    stream_type: StreamType,
    AppJson(input): AppJson<CodeRunnerInput>,
) -> Result<StreamResponse<impl Stream<Item = CodeRunnerChunk>>, AppError> {
    let stream = state.runner.execute(input).await?;

    Ok(StreamResponse::new(stream, stream_type))
}
