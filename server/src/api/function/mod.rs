use aide::axum::ApiRouter;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::AppState;

mod create;
mod run;
mod update;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", create::route())
        .api_route("/{name}", update::route())
        .api_route("/{name}/run", run::route())
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FunctionNamePath {
    name: String,
}
