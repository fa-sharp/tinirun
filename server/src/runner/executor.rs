//! Executor for running code in Docker containers

use std::time::Duration;

use bollard::{
    Docker,
    query_parameters::{
        AttachContainerOptionsBuilder, BuildImageOptionsBuilder, CreateImageOptionsBuilder,
    },
};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::runner::structs::CodeRunnerChunk;

const DEFAULT_TIMEOUT_SECONDS: u32 = 30;
const DEFAULT_MEM_LIMIT_MB: u32 = 512;
const DEFAULT_CPU_LIMIT: f32 = 0.5;

mod build;
mod create;
mod output_task;
mod pull;

pub struct DockerExecutor {
    pub client: Docker,
}

impl DockerExecutor {
    pub fn new(client: Docker) -> Self {
        Self { client }
    }

    pub async fn run(
        &self,
        run_id: &str,
        input: super::CodeRunnerInput,
        dockerfile: String,
        lang_data: super::LanguageData,
        tx: mpsc::Sender<super::CodeRunnerChunk>,
    ) {
        // Code execution config
        let super::CodeRunnerInput {
            code,
            timeout,
            mem_limit_mb,
            ..
        } = input;
        let super::LanguageData {
            image,
            command,
            main_file,
            ..
        } = lang_data;

        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS);
        let mem_limit_mb = mem_limit_mb.unwrap_or(DEFAULT_MEM_LIMIT_MB);

        // Check if base image exists locally, and pull if needed
        send_info(&tx, format!("Checking base image '{image}'...")).await;
        match self.client.inspect_image(&image).await {
            Ok(_) => {}
            Err(bollard::errors::Error::DockerResponseServerError { status_code, .. })
                if status_code == 404 =>
            {
                send_info(&tx, format!("Pulling base image '{image}'...")).await;
                let image_options = CreateImageOptionsBuilder::new().from_image(&image).build();
                let pull_stream = self.client.create_image(Some(image_options), None, None);
                if let Err(err) = pull::process_pull_stream(pull_stream, &tx).await {
                    send_error(&tx, format!("Error pulling image: {err}")).await;
                    return;
                }
            }
            Err(err) => {
                send_error(&tx, format!("Unexpected error when checking image: {err}")).await;
                return;
            }
        }

        // Create build context (Dockerfile and code files)
        send_info(&tx, "Creating build context with 2 files...".into()).await;
        let build_context = build::create_build_context(code, main_file, dockerfile).await;

        // Build Docker image
        send_info(&tx, format!("Building image '{run_id}'...")).await;
        let build_stream = self.client.build_image(
            BuildImageOptionsBuilder::new().t(&run_id).build(),
            None,
            Some(bollard::body_try_stream(build_context)),
        );
        if let Some(image_id) = build::process_build_stream(build_stream, &tx).await {
            send_info(&tx, format!("Built image '{run_id}' with ID {image_id}")).await;
        } else {
            send_error(&tx, format!("Failed to build image '{run_id}'")).await;
            return;
        }

        // Create the container
        let (body, options) =
            create::setup_container(&run_id, &command, timeout, mem_limit_mb, DEFAULT_CPU_LIMIT);
        if let Err(err) = self.client.create_container(Some(options), body).await {
            send_error(&tx, format!("Failed to create container '{run_id}': {err}")).await;
            return;
        }

        // Attach to container and setup capturing of logs/output
        let attach_options = AttachContainerOptionsBuilder::new()
            .stream(true)
            .stdout(true)
            .stderr(true)
            .logs(true)
            .build();
        let container_output_task = match self
            .client
            .attach_container(&run_id, Some(attach_options))
            .await
        {
            Ok(attached) => {
                let task = output_task::attach_and_process_output(attached, timeout, tx.clone());
                tokio::spawn(task)
            }
            Err(err) => {
                send_error(&tx, format!("Failed to attach to container: {err}")).await;
                return;
            }
        };

        // Start container
        send_info(&tx, format!("Starting container with '{command}'...")).await;
        if let Err(e) = self.client.start_container(&run_id, None).await {
            let message = format!("Failed to start container '{run_id}': {e}");
            send_error(&tx, message.clone()).await;
            return;
        }

        // Wait for container to exit, then get exit status and final stdout and stderr
        let container_exit_result = tokio::time::timeout(
            Duration::from_secs(timeout.into()),
            self.client.wait_container(&run_id, None).next(),
        )
        .await;
        let (stdout, stderr) = container_output_task.await.unwrap_or_default();

        let result_chunk = CodeRunnerChunk::Result {
            stdout,
            stderr,
            timeout: container_exit_result.is_err(),
            exit_code: match container_exit_result {
                Ok(Some(Ok(res))) => Some(res.status_code),
                Ok(Some(Err(err))) => match err {
                    bollard::errors::Error::DockerContainerWaitError { code, .. } => Some(code),
                    _ => None,
                },
                _ => None,
            },
        };
        let _ = tx.send(result_chunk).await;
    }
}

// Logging utilities
async fn send_info(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Info(message)).await;
}
async fn send_debug(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Debug(message)).await;
}
async fn send_error(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Error(message)).await;
}
