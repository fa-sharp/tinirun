use std::collections::HashMap;

use fred::prelude::{ClientLike, FredResult, HashesInterface};

use crate::{
    errors::AppError,
    redis::{
        FunctionDetail,
        structs::{FUNCTION_INFO_KEYS, FunctionInfo, FunctionStatus},
    },
};

#[derive(Clone)]
pub struct RedisClient {
    client: fred::prelude::Pool,
    prefix: String,
}

impl RedisClient {
    pub fn new(client: fred::clients::Pool, prefix: &str) -> Self {
        Self {
            client,
            prefix: prefix.to_owned(),
        }
    }

    fn key(&self, name: &str) -> String {
        format!("{}fn:{name}", self.prefix)
    }

    pub async fn get_fn_detail(&self, name: &str) -> Result<Option<FunctionDetail>, AppError> {
        let key = self.key(name);
        if let Some(info) = self
            .client
            .hgetall::<Option<HashMap<_, _>>, _>(&key)
            .await?
        {
            Ok(Some(FunctionDetail::try_from(info)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_fn_info(&self, name: &str) -> Result<Option<FunctionInfo>, AppError> {
        let key = self.key(name);
        let info_values = self
            .client
            .hmget::<Vec<Option<String>>, _, _>(&key, FUNCTION_INFO_KEYS)
            .await?;
        if info_values.first().is_none_or(|v| v.is_none()) {
            return Ok(None);
        }

        let hash: HashMap<String, Option<String>> = HashMap::from_iter(
            FUNCTION_INFO_KEYS
                .into_iter()
                .zip(info_values)
                .map(|(&key, val)| (key.to_owned(), val)),
        );
        Ok(Some(FunctionInfo::try_from(hash)?))
    }

    pub async fn set_fn(&self, name: &str, info: FunctionDetail) -> FredResult<()> {
        self.client
            .hset(self.key(name), HashMap::try_from(info)?)
            .await
    }

    pub async fn set_fn_status(&self, name: &str, status: FunctionStatus) -> FredResult<()> {
        self.client
            .hset(self.key(name), ("status", status.as_ref()))
            .await
    }

    pub async fn shutdown(&self) -> FredResult<()> {
        self.client.quit().await
    }
}
