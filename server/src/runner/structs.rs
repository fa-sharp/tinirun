use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Options for the code runner
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeRunnerInput {
    /// The code to run
    pub code: String,
    /// Language of the code
    pub lang: CodeLanguage,
    /// Dependencies for the code execution
    pub dependencies: Option<Vec<String>>,
    /// Timeout for the code execution in seconds
    pub timeout: Option<u32>,
    /// Memory limit for the code execution in megabytes
    pub mem_limit_mb: Option<u32>,
}

/// Supported languages for the code runner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CodeLanguage {
    Bash,
    Go,
    JavaScript,
    Python,
    Rust,
    TypeScript,
}

/// Config data for each language
#[derive(Debug, Clone, Deserialize)]
pub struct LanguageData {
    pub image: String,
    pub command: String,
    pub template: String,
    pub main_file: String,
}

/// Chunk of the code runner stream output
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", content = "data", rename_all = "lowercase")]
pub enum CodeRunnerChunk {
    /// Info log
    Info(String),
    /// Debug log
    Debug(String),
    /// Error log
    Error(String),
    /// Stdout log from the container
    Stdout(String),
    /// Stderr log from the container
    Stderr(String),
    /// Full result of the code execution
    Result {
        stdout: String,
        stderr: String,
        exit_code: Option<i64>,
        timeout: bool,
    },
}
