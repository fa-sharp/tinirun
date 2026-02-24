use axum_app_wrapper::App;

use crate::config::AppConfig;

mod api;
mod auth;
mod config;
mod errors;
mod input;
mod redis;
mod responses;
mod runner;
mod state;

pub async fn create_app() -> anyhow::Result<(axum::Router, AppConfig, impl Future + Send)> {
    let (router, state, on_shutdown) = App::new()
        .register(config::plugin()) // Extract configuration and add it to state
        .register(redis::plugin()) // Connect to Redis and add Redis client to state
        .register(runner::plugin()) // Connect to Docker and add code runner service to state
        .register(api::plugin()) // Add API routes
        .init()
        .await?;
    let app_config = state.config.to_owned();

    Ok((router.with_state(state), app_config, on_shutdown))
}
