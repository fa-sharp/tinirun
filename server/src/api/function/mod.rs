use aide::axum::ApiRouter;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::AppState;

mod create;
mod get;
mod get_detail;
mod list;
mod run;
mod run_stream;
mod update;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", create::route())
        .api_route("/", list::route())
        .api_route("/{name}/info", get::route())
        .api_route("/{name}/detail", get_detail::route())
        .api_route("/{name}", update::route())
        .api_route("/{name}/run", run::route())
        .api_route("/{name}/run/stream", run_stream::route())
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FunctionNamePath {
    name: String,
}
