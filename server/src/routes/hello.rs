
use axum::extract::{Json, Query};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::AppState;


pub fn routes() -> aide::axum::ApiRouter<AppState> {
    aide::axum::ApiRouter::new()
        .api_route(
            "/",
            aide::axum::routing::get_with(hello_handler, |op| op.summary("Greet user")),
        )
        .api_route(
            "/",
            aide::axum::routing::post_with(post_handler, |op| op.summary("Relay message")),
        )
}

async fn hello_handler(Query(query): Query<HelloQuery>) -> String {
    format!("Hello, {}!", query.name)
}

async fn post_handler(Json(body): Json<PostBody>) -> String {
    format!("Received message: {}", body.message)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct HelloQuery {
    /// The name of the person to greet
    name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PostBody {
    /// The message to relay
    message: String,
}

