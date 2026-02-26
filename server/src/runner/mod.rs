//! Code runner and server plugin

use std::collections::HashMap;

use futures::Stream;
use tinirun_models::{CodeRunnerChunk, CodeRunnerInput, CodeRunnerLanguage, RunFunctionInput};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    errors::AppError,
    redis::{FunctionDetail, FunctionInfo, FunctionStatus, RedisClient},
    runner::{
        constants::{SET_BUILD_ID, SET_USER_AND_HOME_DIR, UID_GID},
        executor::DockerExecutor,
        functions::FunctionExecutor,
        helpers::log,
        structs::{LanguageData, LanguageTemplates},
    },
};

mod constants;
mod executor;
mod functions;
mod helpers;
mod plugin;
mod structs;
mod validators;

pub use plugin::plugin;
pub use validators::validate_deps_input;

const CHANNEL_BUFFER_SIZE: usize = 1024;

/// # Code runner using Docker containers
///
/// Containers are created and destroyed for each execution.
/// Security precautions are taken to ensure that the containers are as isolated and secure as possible - however,
/// there are always risks associated with running untrusted code in Docker.
pub struct DockerRunner {
    client: bollard::Docker,
    redis: RedisClient,
    language_data: HashMap<CodeRunnerLanguage, LanguageData>,
    pub templates: HashMap<CodeRunnerLanguage, LanguageTemplates>,
}

impl DockerRunner {
    pub fn new(
        client: bollard::Docker,
        redis: RedisClient,
        language_data: HashMap<CodeRunnerLanguage, LanguageData>,
        templates: HashMap<CodeRunnerLanguage, LanguageTemplates>,
    ) -> Self {
        Self {
            client,
            redis,
            language_data,
            templates,
        }
    }

    /// Execute the given code in a Docker container and return a stream of events.
    /// Returns an error immediately if the Docker service was unreachable, or the
    /// Dockerfile was unable to be rendered.
    pub async fn execute(
        &self,
        input: CodeRunnerInput,
    ) -> Result<impl Stream<Item = CodeRunnerChunk> + use<>, AppError> {
        // Validate dependency names
        if let Some(deps) = &input.dependencies {
            validators::validate_deps_input(deps).map_err(AppError::BadRequest)?;
        }

        // Render the Dockerfile
        let (lang_data, templates) = self.get_lang_info(&input.lang)?;
        let dockerfile_vars = liquid::object!({
            "image": lang_data.image,
            "main_file": lang_data.main_filename,
            "dependencies": input.dependencies.as_ref().map(|deps| deps.join(" ")),
            "uid_gid": &UID_GID,
            "set_build_id": &SET_BUILD_ID,
            "set_user_and_home_dir": &SET_USER_AND_HOME_DIR,
        });
        let dockerfile = templates
            .dockerfile
            .render(&dockerfile_vars)
            .map_err(|err| AppError::Server(format!("failed to render Dockerfile: {err}")))?;

        // Ping the Docker service to ensure it is reachable
        self.client.ping().await?;

        // Spawn a task to run the code in a Docker container and send back events
        let (tx, rx) = mpsc::channel::<CodeRunnerChunk>(CHANNEL_BUFFER_SIZE);
        let client = self.client.clone();
        tokio::spawn(async move {
            let executor = DockerExecutor::new(client);
            let run_id = Self::gen_run_id();

            tracing::info!("Starting code execution with ID '{run_id}'");
            tokio::select! {
                res = executor.run(&run_id, input, dockerfile, lang_data, tx.clone()) => {
                    if let Err(err) = res {
                        log::send_error(&tx, err).await;
                    }
                    tracing::info!("Code execution '{run_id}' completed");
                }
                _ = tx.closed() => {
                    tracing::info!("Code execution '{run_id}' cancelled (connection dropped)");
                }
            }
            helpers::run_cleanup(&executor.client, &run_id).await;
        });

        // Return the stream of events from the code runner
        Ok(ReceiverStream::new(rx))
    }

    /// Build the function image
    pub async fn build_function(
        &self,
        name: &str,
        info: FunctionDetail,
    ) -> Result<impl Stream<Item = CodeRunnerChunk> + use<>, AppError> {
        let (lang_data, templates) = self.get_lang_info(&info.lang)?;

        // Render the Dockerfile
        let dockerfile_vars = liquid::object!({
            "image": lang_data.image,
            "main_file": lang_data.main_filename,
            "fn_file": lang_data.fn_filename,
            "dependencies": info.dependencies,
            "uid_gid": &UID_GID,
            "set_build_id": &SET_BUILD_ID,
            "set_user_and_home_dir": &SET_USER_AND_HOME_DIR,
        });
        let dockerfile = templates
            .dockerfile
            .render(&dockerfile_vars)
            .map_err(|err| AppError::Server(format!("Failed to render Dockerfile: {err}")))?;

        // Ping the Docker service to ensure it is reachable
        self.client.ping().await?;

        // Spawn a task to build the function image and send back events
        let (tx, rx) = mpsc::channel::<CodeRunnerChunk>(CHANNEL_BUFFER_SIZE);
        let client = self.client.clone();
        let redis = self.redis.clone();
        let name = name.to_owned();
        let main_code = templates.main_file.to_owned();
        tokio::spawn(async move {
            // Build the function and update its status
            let executor = FunctionExecutor::new(client);
            let status = match executor
                .build_fn(&name, info, lang_data, dockerfile, main_code, tx.clone())
                .await
            {
                Ok((tag, id)) => FunctionStatus::Ready { tag, id },
                Err(err) => {
                    log::send_error(&tx, err.clone()).await;
                    FunctionStatus::Error(err)
                }
            };
            if let Err(err) = redis.set_fn_status(&name, status).await {
                tracing::error!("Failed to set status of '{name}' function in Redis: {err}");
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    /// Run the function with the given inputs
    pub async fn run_function(
        &self,
        name: String,
        fn_info: FunctionInfo,
        input: RunFunctionInput,
    ) -> Result<impl Stream<Item = CodeRunnerChunk> + use<>, AppError> {
        let (lang_data, _) = self.get_lang_info(&fn_info.lang)?;

        // Ping the Docker service to ensure it is reachable
        self.client.ping().await?;

        // Spawn a task to run the function in a Docker container and send back events
        let (tx, rx) = mpsc::channel::<CodeRunnerChunk>(CHANNEL_BUFFER_SIZE);
        let client = self.client.clone();
        tokio::spawn(async move {
            let executor = FunctionExecutor::new(client);
            let run_id = Self::gen_run_id();

            tracing::info!("Running function '{name}' with run ID '{run_id}'");
            tokio::select! {
                res = executor.run_function(&run_id, &name, input, lang_data, tx.clone()) => {
                    if let Err(err) = res {
                        log::send_error(&tx, err).await;
                    }
                    tracing::info!("Code execution '{run_id}' completed");
                }
                _ = tx.closed() => {
                    tracing::info!("Code execution '{run_id}' cancelled (connection dropped)");
                }
            }
            helpers::run_cleanup(&executor.client, &run_id).await;
        });

        Ok(ReceiverStream::new(rx))
    }

    fn gen_run_id() -> String {
        format!("code-runner-{}", uuid::Uuid::new_v4())
    }

    fn get_lang_info(
        &self,
        lang: &CodeRunnerLanguage,
    ) -> Result<(LanguageData, &LanguageTemplates), AppError> {
        let lang_data = self
            .language_data
            .get(lang)
            .ok_or_else(|| AppError::Server("Language data not found".into()))?
            .to_owned();
        let templates = self
            .templates
            .get(lang)
            .ok_or_else(|| AppError::Server("Dockerfile template not found".into()))?;

        Ok((lang_data, templates))
    }
}
