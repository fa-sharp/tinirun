use anyhow::anyhow;
use axum_app_wrapper::AdHocPlugin;

use crate::{config::AppConfig, state::AppState};

mod client;
mod structs;

pub use client::RedisClient;
pub use structs::{FunctionDetail, FunctionInfo, FunctionStatus};

/// Initialize the Redis client and add to Axum state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_init(|mut state| async {
        let app_config = state
            .get::<AppConfig>()
            .ok_or_else(|| anyhow!("app config not found"))?;
        let client = RedisClient::new(&app_config.redis_url, "tinirun:").await?;

        state.insert(client);
        Ok(state)
    })
}
