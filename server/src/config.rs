use std::net::{IpAddr, Ipv4Addr};

use anyhow::Context;
use axum_app_wrapper::AdHocPlugin;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AppConfig {
    /// API key that needs to be provided in the `X-Runner-Api-Key` header.
    pub api_key: String,
    /// Interval in seconds between image cleanup runs.
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u32,

    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}
fn default_cleanup_interval() -> u32 {
    300
}
fn default_host() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}
fn default_port() -> u16 {
    8082
}
fn default_log_level() -> String {
    "warn".to_string()
}

/// Plugin that reads and validates configuration, and adds it to server state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_init(|mut state| async move {
        let config = extract_config()?;
        state.insert(config);
        Ok(state)
    })
}

/// Extract the configuration from env variables prefixed with `RUNNER_`.
fn extract_config() -> anyhow::Result<AppConfig> {
    let config = figment::Figment::new()
        .merge(figment::providers::Env::prefixed("RUNNER_"))
        .extract::<AppConfig>()
        .context("Failed to extract valid configuration")?;

    Ok(config)
}
