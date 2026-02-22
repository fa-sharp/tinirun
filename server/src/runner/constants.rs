/// Common Dockerfile instructions for setting up the non-root user and home directory
pub const SET_USER_AND_HOME_DIR: &str = r#"
RUN mkdir -p /app && chown 1000:1000 /app
USER 1000:1000
RUN mkdir -p /tmp/home
WORKDIR /app
"#;
