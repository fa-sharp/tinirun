use axum::{
    Json,
    extract::State,
    response::{Sse, sse::Event},
};
use futures::{Stream, StreamExt};

use crate::{runner::CodeRunnerInput, state::AppState};

pub fn route() -> axum::routing::MethodRouter<AppState> {
    axum::routing::post(handler)
}

async fn handler(
    State(state): State<AppState>,
    Json(input): Json<CodeRunnerInput>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, String> {
    let stream = state
        .runner
        .execute(input)
        .await
        .map_err(|err| err.to_string())?
        .map(|chunk| Event::default().json_data(chunk));

    Ok(Sse::new(stream))
}
