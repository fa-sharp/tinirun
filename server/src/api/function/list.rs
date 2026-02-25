use aide::axum::routing::ApiMethodRouter;
use axum::{Json, extract::State};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{errors::AppError, redis::FunctionInfo, state::AppState};

pub fn route() -> ApiMethodRouter<AppState> {
    aide::axum::routing::get_with(handler, |op| {
        op.id("list_functions")
            .summary("List functions")
            .description("List each function's info and status")
    })
}

async fn handler(State(state): State<AppState>) -> Result<Json<Vec<FunctionItem>>, AppError> {
    let functions = state.redis.list_functions(100).await?;
    Ok(Json(
        functions
            .into_iter()
            .map(|(name, info)| FunctionItem { name, info })
            .collect(),
    ))
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
struct FunctionItem {
    name: String,
    info: FunctionInfo,
}
