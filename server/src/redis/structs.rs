use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use strum::AsRefStr;
use tinirun_models::{CodeRunnerError, CodeRunnerLanguage, UpdateFunctionInput};

/// Build status of a function
#[derive(Debug, Default, Clone, AsRefStr, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FunctionStatus {
    #[default]
    NotBuilt,
    Building,
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
        serde_json::from_value(serde_json::to_value(hash)?)
    }
}

impl TryFrom<FunctionDetail> for HashMap<String, Option<String>> {
    type Error = serde_json::Error;
    fn try_from(info: FunctionDetail) -> Result<Self, serde_json::Error> {
        serde_json::from_value(serde_json::to_value(info)?)
    }
}

/// Selected function info stored in Redis
#[serde_as]
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

impl TryFrom<HashMap<String, Option<String>>> for FunctionInfo {
    type Error = serde_json::Error;
    fn try_from(hash: HashMap<String, Option<String>>) -> Result<Self, serde_json::Error> {
        serde_json::from_value(serde_json::to_value(hash)?)
    }
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
