//! Shared models for the tinirun API

use std::path::{Component, Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use validator::{Validate, ValidationError};

/// Supported languages for the code runner
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CodeRunnerLanguage {
    Bash,
    Go,
    #[default]
    JavaScript,
    Python,
    Rust,
    TypeScript,
}

/// Options for the code runner
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CodeRunnerInput {
    /// The code to run
    pub code: String,
    /// Language of the code
    pub lang: CodeRunnerLanguage,
    /// Dependencies for the code execution. Versions and features can be specified
    /// depending on the language's package manager.
    #[schemars(example = vec!["lodash"])]
    #[schemars(example = vec!["serde=1.0", "tokio=1.0", "--features", "serde/derive"])]
    pub dependencies: Option<Vec<String>>,
    /// Additional files for the code execution. These files will be available to the code
    /// under the `./files` directory.
    #[validate(length(max = 50))]
    pub files: Option<Vec<CodeRunnerFile>>,
    /// Timeout for the code execution in seconds
    #[validate(range(min = 5, max = 300))]
    pub timeout: Option<u32>,
    /// Memory limit for the code execution in megabytes
    #[validate(range(min = 1, max = 2048))]
    pub mem_limit_mb: Option<u32>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CodeRunnerFile {
    /// Path of the file relative to the `./files` directory. Must be
    /// a relative path, and cannot contain `..` or `.`
    #[validate(custom(function = "validate_path"))]
    #[schemars(example = "file.txt")]
    #[schemars(example = "foo/file.txt")]
    pub path: PathBuf,
    /// Base64 encoded content of the file
    #[serde_as(as = "Base64")]
    #[schemars(with = "String")]
    pub content: Vec<u8>,
}
fn validate_path(path: &PathBuf) -> Result<(), ValidationError> {
    if path.is_absolute() {
        Err(ValidationError::new("absolute_path").with_message("file path must be relative".into()))
    } else if path
        .components()
        .any(|c| c == Component::ParentDir || c == Component::CurDir)
    {
        Err(ValidationError::new("invalid_path")
            .with_message("file path must not have '.' or '..'".into()))
    } else if path == &Path::new("Dockerfile") || path == &Path::new("./Dockerfile") {
        Err(ValidationError::new("dockerfile").with_message("cannot provide Dockerfile".into()))
    } else {
        Ok(())
    }
}

/// Chunk of the code runner stream output
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum CodeRunnerChunk {
    /// # Info
    /// Streamed info logs
    Info(String),
    /// # Debug
    /// Streamed debug logs
    Debug(String),
    /// # Stdout
    /// Streamed stdout log from the container
    Stdout(String),
    /// # Stderr
    /// Streamed stderr log from the container
    Stderr(String),
    /// # Error
    /// This represents an issue that occurred before code could be executed. This
    /// should be the final chunk of the stream.
    Error(String),
    /// # Build error
    /// This represents an issue that occurred during the building of the container. This
    /// should be the final chunk of the stream.
    BuildError { message: String, build_logs: String },
    /// # Execution result
    /// Full result of the code execution. This should be the final chunk of the stream.
    Result {
        stdout: String,
        stderr: String,
        exit_code: Option<i64>,
        timeout: bool,
    },
}
