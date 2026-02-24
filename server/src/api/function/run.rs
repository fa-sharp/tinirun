use aide::axum::routing::ApiMethodRouter;
use axum::extract::{Path, State};
use tinirun_models::{CodeRunnerChunk, RunFunctionInput};

use crate::{
    api::function::FunctionNamePath,
    errors::AppError,
    input::{AppJson, StreamType},
    redis::FunctionStatus,
    responses::StreamResponse,
    state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("run_function")
            .summary("Run function")
            .description("Run a saved function with the given inputs")
    })
}

async fn handler(
    State(state): State<AppState>,
    stream_type: StreamType,
    Path(FunctionNamePath { name }): Path<FunctionNamePath>,
    AppJson(input): AppJson<RunFunctionInput>,
) -> Result<StreamResponse<impl futures::Stream<Item = CodeRunnerChunk>>, AppError> {
    let fn_info = state
        .redis
        .get_fn_info(&name)
        .await?
        .ok_or(AppError::NotFound)?;
    if fn_info.status != FunctionStatus::Ready {
        return Err(AppError::BadRequest("Function not ready".to_owned()));
    }

    let stream = state.runner.run_function(name, fn_info, input).await?;
    Ok(StreamResponse::new(stream, stream_type))
}
