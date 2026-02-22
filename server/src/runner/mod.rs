//! Code runner and server plugin

use std::collections::HashMap;

use anyhow::Context;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::runner::{
    constants::SET_USER_AND_HOME_DIR,
    executor::DockerExecutor,
    structs::{CodeRunnerChunk, LanguageData},
};

mod cleanup;
mod constants;
mod executor;
mod plugin;
mod structs;
mod validators;

pub use plugin::plugin;
pub use structs::{CodeLanguage, CodeRunnerInput};

/// # Code runner using Docker containers
///
/// Containers are created and destroyed for each execution.
/// Security precautions are taken to ensure that the containers are as isolated and secure as possible - however,
/// there are always risks associated with running untrusted code in Docker.
pub struct DockerRunner {
    client: bollard::Docker,
    language_data: HashMap<CodeLanguage, LanguageData>,
    dockerfile_templates: HashMap<String, liquid::Template>,
}

impl DockerRunner {
    pub fn new(
        client: bollard::Docker,
        language_data: HashMap<CodeLanguage, LanguageData>,
        dockerfile_templates: HashMap<String, liquid::Template>,
    ) -> Self {
        Self {
            client,
            language_data,
            dockerfile_templates,
        }
    }

    /// Execute the given code in a Docker container and return a stream of events.
    /// Returns an error immediately if the Docker service was unreachable, or the
    /// Dockerfile was unable to be rendered.
    pub async fn execute(
        &self,
        input: CodeRunnerInput,
    ) -> anyhow::Result<impl Stream<Item = CodeRunnerChunk> + use<>> {
        // Validate dependency names
        if let Some(deps) = &input.dependencies {
            validators::validate_deps_input(deps)?;
        }

        // Render the Dockerfile
        let lang_data = self
            .language_data
            .get(&input.lang)
            .context("language data not found")?
            .to_owned();
        let dockerfile_template = self
            .dockerfile_templates
            .get(&lang_data.template)
            .context("Dockerfile template not found")?;
        let dockerfile_vars = liquid::object!({
            "image": lang_data.image,
            "main_file": lang_data.main_file,
            "dependencies": input.dependencies.as_ref().map(|deps| deps.join(" ")),
            "set_user_and_home_dir": &SET_USER_AND_HOME_DIR,
        });
        let dockerfile = dockerfile_template
            .render(&dockerfile_vars)
            .context("failed to render Dockerfile")?;

        // Ping the Docker service to ensure it is reachable
        self.client.ping().await.context("could not reach Docker")?;

        // Spawn a task to run the code in a Docker container and send back events
        let (tx, rx) = mpsc::channel::<CodeRunnerChunk>(1024);
        let client = self.client.clone();
        tokio::spawn(async move {
            let executor = DockerExecutor::new(client);
            let run_id = format!("code-runner-{}", uuid::Uuid::new_v4());
            tracing::info!("Starting code execution with ID '{run_id}'");
            tokio::select! {
                _ = executor.run(&run_id, input, dockerfile, lang_data, tx.clone()) => {
                    tracing::info!("Code execution '{run_id}' completed");
                }
                _ = tx.closed() => {
                    tracing::info!("Code execution '{run_id}' cancelled (connection dropped)");
                }
            }
            cleanup::docker_cleanup(&executor.client, &run_id).await;
        });

        // Return the stream of events from the code runner
        Ok(ReceiverStream::new(rx))
    }
}
