use aide::axum::routing::ApiMethodRouter;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    api::{ApiTag, function::FunctionNamePath},
    errors::AppError,
    redis::FunctionInfo,
    state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::get_with(handler, |op| {
        op.id("get_function")
            .tag(ApiTag::Functions.into())
            .summary("Get function info")
            .description("Get function info and status")
    })
}

async fn handler(
    State(state): State<AppState>,
    Path(FunctionNamePath { name }): Path<FunctionNamePath>,
) -> Result<Json<FunctionInfo>, AppError> {
    match state.redis.get_fn_info(&name).await? {
        Some(function) => Ok(Json(function)),
        None => Err(AppError::NotFound),
    }
}
