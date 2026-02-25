use aide::axum::routing::ApiMethodRouter;
use axum::{
    Json,
    extract::{Path, State},
};
use futures::StreamExt;
use schemars::JsonSchema;
use serde::Serialize;
use tinirun_models::{CodeRunnerChunk, RunFunctionInput};

use crate::{
    api::function::FunctionNamePath, errors::AppError, input::AppJson, redis::FunctionStatus,
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
    Path(FunctionNamePath { name }): Path<FunctionNamePath>,
    AppJson(input): AppJson<RunFunctionInput>,
) -> Result<Json<RunFunctionOutput>, AppError> {
    let fn_info = state
        .redis
        .get_fn_info(&name)
        .await?
        .ok_or(AppError::NotFound)?;
    if !matches!(fn_info.status, FunctionStatus::Ready { .. }) {
        return Err(AppError::BadRequest("Function not ready".to_owned()));
    }

    let mut stream = state.runner.run_function(name, fn_info, input).await?;
    while let Some(chunk) = stream.next().await {
        match chunk {
            CodeRunnerChunk::Error(err) => return Err(AppError::ExecutionFailed(err)),
            CodeRunnerChunk::Result {
                stdout,
                stderr,
                exit_code,
                timeout,
            } => {
                return Ok(Json(RunFunctionOutput {
                    stdout,
                    stderr,
                    exit_code,
                    timeout,
                }));
            }
            _ => {}
        }
    }

    Err(AppError::Server("No result/error from function".to_owned()))
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
struct RunFunctionOutput {
    stdout: String,
    stderr: String,
    exit_code: Option<i64>,
    timeout: bool,
}
