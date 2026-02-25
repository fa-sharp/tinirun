use aide::axum::routing::ApiMethodRouter;
use axum::extract::{Path, State};
use futures::Stream;
use tinirun_models::{CodeRunnerChunk, UpdateFunctionInput};

use crate::{
    api::{ApiTag, function::FunctionNamePath},
    errors::AppError,
    input::{AppJson, StreamType},
    responses::StreamResponse,
    runner::validate_deps_input,
    state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("update_function")
            .tag(ApiTag::Functions.into())
            .summary("Update function")
            .description("Modify a saved function")
    })
}

async fn handler(
    State(state): State<AppState>,
    stream_type: StreamType,
    Path(FunctionNamePath { name }): Path<FunctionNamePath>,
    AppJson(input): AppJson<UpdateFunctionInput>,
) -> Result<StreamResponse<impl Stream<Item = CodeRunnerChunk>>, AppError> {
    if let Some(dependencies) = &input.dependencies {
        validate_deps_input(dependencies)
            .map_err(|e| AppError::BadRequest(format!("Invalid dependencies: {e}")))?;
    }

    let mut fn_detail = state
        .redis
        .get_fn_detail(&name)
        .await?
        .ok_or(AppError::NotFound)?;
    fn_detail.update(input);

    let build_stream = state
        .runner
        .build_function(&name, fn_detail.clone())
        .await?;
    state.redis.set_fn(&name, fn_detail).await?;

    Ok(StreamResponse::new(build_stream, stream_type))
}
