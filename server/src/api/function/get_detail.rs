use aide::axum::routing::ApiMethodRouter;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    api::function::FunctionNamePath, errors::AppError, redis::FunctionDetail, state::AppState,
};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::get_with(handler, |op| {
        op.id("get_function_detail")
            .summary("Get function details")
            .description("Get full function details and code")
    })
}

pub async fn handler(
    State(state): State<AppState>,
    Path(FunctionNamePath { name }): Path<FunctionNamePath>,
) -> Result<Json<FunctionDetail>, AppError> {
    match state.redis.get_fn_detail(&name).await? {
        Some(function) => Ok(Json(function)),
        None => Err(AppError::NotFound),
    }
}
