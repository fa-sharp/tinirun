use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use tinirun_models::CodeRunnerLanguage;

/// Full function info stored in Redis
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionDetail {
    pub code: String,
    pub lang: CodeRunnerLanguage,
    pub description: Option<String>,
    pub dependencies: Option<String>,
    pub status: FunctionStatus,
    pub version: u32,
}

/// Selected function info stored in Redis Redis
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionInfo {
    pub lang: CodeRunnerLanguage,
    pub description: Option<String>,
    pub status: FunctionStatus,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Keys in FunctionInfo (e.g. to fetch via `HMGET` from Redis)
pub const FUNCTION_INFO_KEYS: &[&str; 6] = &[
    "lang",
    "description",
    "status",
    "version",
    "created_at",
    "updated_at",
];

#[derive(Debug, Default, Clone, PartialEq, AsRefStr, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FunctionStatus {
    #[default]
    NotBuilt,
    Building,
    Ready,
}

impl From<FunctionDetail> for HashMap<String, String> {
    fn from(info: FunctionDetail) -> Self {
        HashMap::from_iter([
            ("code".into(), info.code),
            ("lang".into(), info.lang.as_ref().to_owned()),
            ("description".into(), info.description.unwrap_or_default()),
            ("dependencies".into(), info.dependencies.unwrap_or_default()),
            ("status".into(), info.status.as_ref().to_owned()),
        ])
    }
}
