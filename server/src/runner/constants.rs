// Container and image labels
/// Label given to all containers and images created by the app
pub const APP_LABEL: &str = "tinirun";
/// Label indicating the run ID that created the container
pub const ID_LABEL: &str = "tinirun-id";
/// Image label for one-off executions
pub const EXEC_LABEL: &str = "tinirun-exec";
/// Image label for function images
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
