use std::{collections::HashMap, time::Duration};

use anyhow::{Context, anyhow};
use axum_app_wrapper::AdHocPlugin;
use bollard::Docker;

use crate::{
    config::AppConfig,
    redis::RedisClient,
    runner::{
        DockerRunner,
        helpers::{image_cleanup_task, sync_function_status_task},
        structs::LanguageTemplates,
    },
    state::AppState,
};

/// Static directory containing language configs and Dockerfile templates
static DOCKER_STATIC_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/docker");

/// Server plugin that runs on startup to connect to Docker, parse language config and Dockerfile templates,
/// and add the code runner service to Axum state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_init(|mut state| async {
        let app_config = state
            .get::<AppConfig>()
            .ok_or_else(|| anyhow!("app config not found"))?;

        // Connect to Docker and initialize client
        let client = tokio::task::spawn_blocking(Docker::connect_with_defaults)
            .await?
            .context("could not connect to Docker")?;

        // Parse language config
        let language_data: HashMap<super::CodeRunnerLanguage, super::structs::LanguageData> = {
            let language_data_file = DOCKER_STATIC_DIR
                .get_file("data.toml")
                .ok_or_else(|| anyhow!("language data file not found"))?;

            toml::from_slice(language_data_file.contents())
                .context("could not parse language data file")?
        };

        // Parse all language templates
        let templates: HashMap<super::CodeRunnerLanguage, LanguageTemplates> = {
            let parser = liquid::ParserBuilder::with_stdlib().build()?;
            let templates_dir = DOCKER_STATIC_DIR
                .get_dir("templates")
                .ok_or_else(|| anyhow!("template directory not found"))?;

            let mut templates = HashMap::new();
            for (lang, lang_data) in language_data.iter() {
                let lang_dir = templates_dir.path().join(&lang_data.template);
                let folder = templates_dir
                    .get_dir(&lang_dir)
                    .ok_or_else(|| anyhow!("missing template directory for {lang:?}"))?;

                let template_file = folder
                    .get_file(lang_dir.join("Dockerfile.liquid"))
                    .ok_or_else(|| anyhow!("missing Dockerfile template for {lang:?}"))?;
                let template = parser
                    .parse(template_file.contents_utf8().ok_or_else(|| {
                        anyhow!("invalid UTF8 in Dockerfile template for {lang:?}")
                    })?)
                    .with_context(|| format!("failed to parse Dockerfile template for {lang:?}"))?;

                let main_file = folder
                    .get_file(lang_dir.join(&lang_data.main_filename))
                    .ok_or_else(|| anyhow!("missing main file for {lang:?}"))?
                    .contents_utf8()
                    .ok_or_else(|| anyhow!("invalid UTF8 in main file for {lang:?}"))?;
                let fn_file = folder
                    .get_file(lang_dir.join(&lang_data.fn_filename))
                    .ok_or_else(|| anyhow!("missing function file for {lang:?}"))?
                    .contents_utf8()
                    .ok_or_else(|| anyhow!("invalid UTF8 in function file for {lang:?}"))?;

                templates.insert(
                    lang.to_owned(),
                    LanguageTemplates {
                        dockerfile: template,
                        main_file: main_file.to_owned(),
                        fn_file: fn_file.to_owned(),
                    },
                );
            }

            templates
        };

        let redis = state
            .get::<RedisClient>()
            .ok_or_else(|| anyhow!("redis not in state"))?
            .to_owned();

        // Start image cleanup task
        let cleanup_period = Duration::from_secs(app_config.cleanup_interval.into());
        tokio::spawn(image_cleanup_task(client.clone(), cleanup_period));

        // Start function status sync task
        tokio::spawn(sync_function_status_task(
            client.clone(),
            redis.clone(),
            Duration::from_secs(120),
        ));

        // Add runner to state
        let runner = DockerRunner::new(client, redis, language_data, templates);
        state.insert(runner);

        Ok(state)
    })
}
