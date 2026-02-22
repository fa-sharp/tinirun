use axum_app_wrapper::App;

use crate::config::AppConfig;

mod config;
mod routes;
mod runner;
mod state;

pub async fn create_app() -> anyhow::Result<(axum::Router, AppConfig, impl Future + Send)> {
    let (router, state, on_shutdown) = App::new()
        .register(config::plugin()) // Extract configuration and add it to state
        .register(runner::plugin()) // Connect to Docker and add code runner service to state
        .register(routes::plugin()) // Add API routes
        .init()
        .await?;
    let app_config = state.config.to_owned();

    Ok((router.with_state(state), app_config, on_shutdown))
}
