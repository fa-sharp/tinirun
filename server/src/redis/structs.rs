use schemars::JsonSchema;
use serde::{Deserialize, Serialize, ser::Error};
use serde_with::{DisplayFromStr, serde_as, skip_serializing_none};
use std::collections::HashMap;
use tinirun_models::{CodeRunnerError, CodeRunnerLanguage, UpdateFunctionInput};

/// Build status of the function
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FunctionStatus {
    /// Function has not been built yet
    #[default]
    NotBuilt,
    /// Function is being built
    Building,
    /// The latest build of the function failed
    Error(CodeRunnerError),
    /// Function is ready to be used
    Ready {
        /// The image tag of the function
        tag: String,
        /// The image id of the function
        id: String,
    },
}

/// Full function info stored in Redis
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionDetail {
    pub code: String,
    pub lang: CodeRunnerLanguage,
    pub description: Option<String>,
    pub dependencies: Option<String>,
    pub status: FunctionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "DisplayFromStr")]
    pub version: u32,
}

impl TryFrom<HashMap<String, String>> for FunctionDetail {
    type Error = serde_json::Error;

    fn try_from(hash: HashMap<String, String>) -> Result<Self, serde_json::Error> {
        let value_hash: serde_json::Map<String, serde_json::Value> = hash
            .into_iter()
            .map(|(key, value_str)| Ok((key, serde_json::from_str(&value_str)?)))
            .collect::<Result<_, _>>()?;
        serde_json::from_value(serde_json::Value::Object(value_hash))
    }
}

impl TryFrom<FunctionDetail> for HashMap<String, String> {
    type Error = serde_json::Error;
    fn try_from(info: FunctionDetail) -> Result<Self, serde_json::Error> {
        match serde_json::to_value(info)? {
            serde_json::Value::Object(map) => map
                .into_iter()
                .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
                .collect(),
            _ => Err(serde_json::Error::custom("FunctionDetail is not an object")),
        }
    }
}

/// Selected function info stored in Redis
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionInfo {
    pub lang: CodeRunnerLanguage,
    pub description: Option<String>,
    pub status: FunctionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "DisplayFromStr")]
    pub version: u32,
}

/// Keys in FunctionInfo (e.g. to fetch via `HMGET` from Redis)
pub const FUNCTION_INFO_KEYS: &[&str; 6] = &[
    "lang",
    "description",
    "status",
    "created_at",
    "updated_at",
    "version",
];

impl TryFrom<HashMap<String, String>> for FunctionInfo {
    type Error = serde_json::Error;
    fn try_from(hash: HashMap<String, String>) -> Result<Self, serde_json::Error> {
        let value_hash: serde_json::Map<String, serde_json::Value> = hash
            .into_iter()
            .map(|(key, value_str)| Ok((key, serde_json::from_str(&value_str)?)))
            .collect::<Result<_, _>>()?;
        serde_json::from_value(serde_json::Value::Object(value_hash))
    }
}

impl FunctionDetail {
    /// Update function details with new input, set status to `Building`,
    /// bump version and `updated_at` timestamp
    pub fn update(&mut self, updated_info: UpdateFunctionInput) {
        self.code = updated_info.code;
        self.description = updated_info.description;
        self.dependencies = updated_info.dependencies.map(|d| d.join(" "));
        self.status = FunctionStatus::Building;
        self.updated_at = chrono::Utc::now();
        self.version += 1;
    }
}
