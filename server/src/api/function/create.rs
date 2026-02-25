use aide::axum::routing::ApiMethodRouter;
use axum::extract::State;
use futures::Stream;
use tinirun_models::{CodeRunnerChunk, CreateFunctionInput};

use crate::{
    errors::AppError,
    input::{AppJson, StreamType},
    redis::{FunctionDetail, FunctionStatus},
    responses::StreamResponse,
    state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::post_with(handler, |op| {
        op.id("create_function")
            .summary("Create function")
            .description("Create a new function")
    })
}

async fn handler(
    State(state): State<AppState>,
    stream_type: StreamType,
    AppJson(input): AppJson<CreateFunctionInput>,
) -> Result<StreamResponse<impl Stream<Item = CodeRunnerChunk>>, AppError> {
    if let Some(_) = state.redis.get_fn_info(&input.name).await? {
        return Err(AppError::BadRequest("Function already exists".into()));
    }

    let templates = state
        .runner
        .templates
        .get(&input.language)
        .ok_or(AppError::Server("Language templates not found".into()))?;
    let fn_detail = FunctionDetail {
        code: templates.fn_file.to_owned(),
        lang: input.language,
        status: FunctionStatus::Building,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        ..Default::default()
    };

    let build_stream = state
        .runner
        .build_function(&input.name, fn_detail.clone())
        .await?;
    state.redis.set_fn(&input.name, fn_detail).await?;

    Ok(StreamResponse::new(build_stream, stream_type))
}
