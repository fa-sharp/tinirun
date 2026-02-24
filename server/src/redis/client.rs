use std::{collections::HashMap, time::Duration};

use fred::prelude::{FredResult, HashesInterface};

use crate::{
    errors::AppError,
    redis::{
        FunctionDetail,
        structs::{FUNCTION_INFO_KEYS, FunctionInfo, FunctionStatus},
    },
};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Clone)]
pub struct RedisClient {
    client: fred::prelude::Pool,
    prefix: String,
}

impl RedisClient {
    pub async fn new(url: &str, prefix: &str) -> Result<Self, anyhow::Error> {
        let config = fred::prelude::Config::from_url(url)?;
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

        Ok(Self {
            client,
            prefix: prefix.to_owned(),
        })
    }

    fn key(&self, name: &str) -> String {
        format!("{}function:{name}", self.prefix)
    }

    pub async fn get_fn_detail(&self, name: &str) -> Result<Option<FunctionDetail>, AppError> {
        let key = self.key(name);
        if let Some(info) = self.client.hgetall::<Option<_>, _>(&key).await? {
            Ok(Some(serde_json::from_value(info)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_fn_info(&self, name: &str) -> Result<Option<FunctionInfo>, AppError> {
        let key = self.key(name);
        if let Some(info) = self
            .client
            .hmget::<Option<_>, _, _>(&key, FUNCTION_INFO_KEYS)
            .await?
        {
            Ok(Some(serde_json::from_value(info)?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_fn(&self, name: &str, info: FunctionDetail) -> FredResult<()> {
        self.client.hset(self.key(name), HashMap::from(info)).await
    }

    pub async fn set_fn_status(&self, name: &str, status: FunctionStatus) -> FredResult<()> {
        self.client
            .hset(self.key(name), ("status", status.as_ref()))
            .await
    }
}
