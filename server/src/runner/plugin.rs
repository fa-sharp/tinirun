use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use axum_app_wrapper::AdHocPlugin;
use bollard::Docker;

use crate::{
    config::AppConfig,
    runner::{DockerRunner, cleanup::image_cleanup},
    state::AppState,
};

/// Static directory containing language configs and Dockerfile templates
static DOCKER_STATIC_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/docker");

/// Server plugin that runs on startup to connect to Docker, parse language config and Dockerfile templates,
/// and add the code runner service to Axum state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_init(|mut state| async {
        let app_config = state.get::<AppConfig>().unwrap();

        // Connect to Docker and initialize client
        let client = tokio::task::spawn_blocking(Docker::connect_with_local_defaults)
            .await?
            .context("could not connect to Docker")?;

        // Parse language config
        let language_data: HashMap<super::CodeLanguage, super::structs::LanguageData> = {
            let language_data_file = DOCKER_STATIC_DIR
                .get_file("data.toml")
                .ok_or_else(|| anyhow::anyhow!("language data file not found"))?;
            toml::from_slice(language_data_file.contents())
                .context("could not parse language data file")?
        };

        // Parse all Dockerfile templates
        let templates: HashMap<String, liquid::Template> = {
            let parser = liquid::ParserBuilder::with_stdlib().build()?;
            let dir = DOCKER_STATIC_DIR
                .get_dir("templates")
                .ok_or_else(|| anyhow::anyhow!("template directory not found"))?;
            dir.files()
                .filter_map(|file| {
                    let lang = file.path().file_stem()?.to_string_lossy().into_owned();
                    let parse_result = parser.parse(file.contents_utf8()?);
                    Some((lang, parse_result))
                })
                .map(|(lang, parse_result)| {
                    let template = parse_result.with_context(|| {
                        format!("Dockerfile template parsing failed: '{lang}.liquid'")
                    })?;
                    Ok((lang, template))
                })
                .collect::<Result<_, anyhow::Error>>()?
        };

        // Start image cleanup task
        let cleanup_period = Duration::from_secs(app_config.cleanup_interval.into());
        tokio::spawn(image_cleanup(client.clone(), cleanup_period));

        // Add runner to state
        let runner = DockerRunner::new(client, language_data, templates);
        state.insert(runner);

        Ok(state)
    })
}
