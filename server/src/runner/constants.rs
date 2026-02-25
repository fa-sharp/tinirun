// Container labels
/// Container label for one-off executions
pub const EXEC_LABEL: &str = "tinirun-exec";
/// Container label for function images
pub const FN_LABEL: &str = "tinirun-fn";

// Dockerfile constants
/// User and group for code execution containers
pub const UID_GID: &str = "1000:1000";
/// Common Dockerfile instructions for setting up the non-root user and home directory
pub const SET_USER_AND_HOME_DIR: &str = r#"
RUN mkdir -p /app && chown 1000:1000 /app
USER 1000:1000
RUN mkdir -p /tmp/home
WORKDIR /app
"#;
