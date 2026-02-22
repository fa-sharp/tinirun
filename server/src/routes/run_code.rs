use axum::{extract::State, response::sse::Event};
use futures::{Stream, StreamExt};

use crate::{
    input::AppValidJson,
    responses::SseResponse,
    runner::{CodeRunnerChunk, CodeRunnerInput},
    state::AppState,
};

pub fn route() -> aide::axum::routing::ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.summary("Run code")
            .description("Run a one-off script with the given parameters")
    })
}

async fn handler(
    State(state): State<AppState>,
    AppValidJson(input): AppValidJson<CodeRunnerInput>,
) -> Result<SseResponse<impl Stream<Item = Result<Event, axum::Error>>, CodeRunnerChunk>, String> {
    let stream = state
        .runner
        .execute(input)
        .await
        .map_err(|err| err.to_string())?
        .map(|chunk| Event::default().json_data(chunk));

    Ok(SseResponse::new(stream))
}
