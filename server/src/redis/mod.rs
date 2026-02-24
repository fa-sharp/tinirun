use std::time::Duration;

use anyhow::{Context, anyhow};
use axum_app_wrapper::AdHocPlugin;
use fred::prelude::ClientLike;

use crate::{config::AppConfig, state::AppState};

mod client;
mod structs;

pub use client::RedisClient;
pub use structs::{FunctionDetail, FunctionInfo, FunctionStatus};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);

/// Initialize the Redis client and add to Axum state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new()
        .on_init(|mut state| async {
            let app_config = state
                .get::<AppConfig>()
                .ok_or_else(|| anyhow!("app config not found"))?;
            let config = fred::prelude::Config::from_url(&app_config.redis_url)
                .context("Invalid Redis URL")?;
            let client = fred::types::Builder::from_config(config)
                .with_connection_config(|config| {
                    config.connection_timeout = CLIENT_TIMEOUT;
                    config.internal_command_timeout = CLIENT_TIMEOUT;
                    config.max_command_attempts = 2;
                    config.tcp = fred::prelude::TcpConfig {
                        nodelay: Some(true),
                        ..Default::default()
                    };
                })
                .set_policy(fred::prelude::ReconnectPolicy::new_linear(0, 10_000, 1000))
                .with_performance_config(|config| {
                    config.default_command_timeout = CLIENT_TIMEOUT;
                })
                .build_pool(4)?;
            client.init().await.context("Failed to connect to Redis")?;

            state.insert(RedisClient::new(client, "tinirun:"));
            Ok(state)
        })
        .on_shutdown(|state| {
            let redis = state.redis.clone();
            async move {
                if let Err(e) = redis.shutdown().await {
                    tracing::error!("Failed to shutdown Redis: {e}");
                }
            }
        })
}
