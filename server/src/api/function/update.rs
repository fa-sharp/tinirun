use aide::axum::routing::ApiMethodRouter;
use axum::extract::{Path, State};
use futures::Stream;
use tinirun_models::{CodeRunnerChunk, UpdateFunctionInput};

use crate::{
    api::function::FunctionNamePath,
    errors::AppError,
    input::{AppJson, StreamType},
    redis::{FunctionDetail, FunctionStatus},
    responses::StreamResponse,
    runner::validate_deps_input,
    state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("update_function")
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

    let curr_info = state
        .redis
        .get_fn_detail(&name)
        .await?
        .ok_or(AppError::NotFound)?;
    let updated_info = FunctionDetail {
        code: input.code,
        lang: curr_info.lang,
        description: input.description,
        dependencies: input.dependencies.map(|d| d.join(" ")),
        status: FunctionStatus::Building,
        version: curr_info.version + 1,
    };
    state.redis.set_fn(&name, updated_info.clone()).await?;

    let stream = state.runner.build_function(name, updated_info).await?;
    Ok(StreamResponse::new(stream, stream_type))
}
