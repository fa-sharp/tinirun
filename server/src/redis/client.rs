use std::collections::HashMap;

use fred::{
    prelude::{ClientLike, FredResult, HashesInterface},
    types::scan::ScanType,
};
use futures::{StreamExt, TryStreamExt};

use crate::redis::{
    FunctionDetail,
    structs::{FUNCTION_INFO_KEYS, FunctionInfo, FunctionStatus},
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

    pub async fn get_fn_detail(&self, name: &str) -> FredResult<Option<FunctionDetail>> {
        let key = self.key(name);
        if let Some(info) = self
            .client
            .hgetall::<Option<HashMap<_, _>>, _>(&key)
            .await?
            && !info.is_empty()
        {
            Ok(Some(FunctionDetail::try_from(info)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_fn_info(&self, name: &str) -> FredResult<Option<FunctionInfo>> {
        let key = self.key(name);
        let info_values = self
            .client
            .hmget::<Vec<Option<String>>, _, _>(&key, FUNCTION_INFO_KEYS)
            .await?;
        if info_values.first().is_none_or(|v| v.is_none()) {
            return Ok(None);
        }

        let hash: HashMap<String, String> = HashMap::from_iter(
            FUNCTION_INFO_KEYS
                .into_iter()
                .zip(info_values)
                .filter_map(|(&key, val)| Some((key.to_owned(), val?))),
        );
        Ok(Some(FunctionInfo::try_from(hash)?))
    }

    pub async fn set_fn(&self, name: &str, info: FunctionDetail) -> FredResult<()> {
        let values = HashMap::try_from(info)?;
        self.client.hset(self.key(name), values).await
    }

    pub async fn set_fn_status(&self, name: &str, status: FunctionStatus) -> FredResult<()> {
        self.client
            .hset(self.key(name), ("status", serde_json::to_string(&status)?))
            .await
    }

    pub async fn list_functions(&self, limit: u32) -> FredResult<Vec<(String, FunctionInfo)>> {
        let keys: Vec<_> = self
            .client
            .next()
            .scan_buffered(self.key("*"), Some(limit), Some(ScanType::Hash))
            .take(limit as usize)
            .try_collect()
            .await?;
        let mut functions = Vec::new();
        for key in keys {
            if let Some(name) = key.as_str().and_then(|s| s.split(':').last()) {
                if let Some(info) = self.get_fn_info(name).await? {
                    functions.push((name.to_owned(), info));
                }
            }
        }

        Ok(functions)
    }

    pub async fn shutdown(&self) -> FredResult<()> {
        self.client.quit().await
    }
}
