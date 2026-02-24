//! Executor for running code in Docker containers

use std::{path::PathBuf, time::Duration};

use bollard::{
    Docker,
    query_parameters::{AttachContainerOptionsBuilder, BuildImageOptionsBuilder},
};
use futures::StreamExt;
use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;

use crate::runner::{
    constants::EXEC_LABEL,
    helpers::{self, log},
};

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
            files,
            timeout,
            mem_limit_mb,
            cpu_limit,
            ..
        } = input;
        let super::LanguageData {
            image,
            command,
            main_filename: main_file,
            ..
        } = lang_data;

        // Check if base image exists locally, and pull if needed
        log::send_info(&tx, format!("Checking base image '{image}'...")).await;
        if let Err(e) = helpers::pull_image(&self.client, &image, &tx).await {
            log::send_error(&tx, format!("Error while checking/pulling image: {e}")).await;
            return;
        }

        // Create build context (Dockerfile, code, attached files)
        let code_files = [
            (PathBuf::from("Dockerfile"), dockerfile.into_bytes()),
            (PathBuf::from(main_file), code.into_bytes()),
        ];
        let attached_files = files
            .unwrap_or_default()
            .into_iter()
            .map(|file| (file.path, file.content));
        let all_files: Vec<_> = code_files.into_iter().chain(attached_files).collect();
        let mut build_ctx_message = String::from("Creating build context:");
        for (path, _) in all_files.iter() {
            build_ctx_message.push_str(&format!(" {path:?}"));
        }
        log::send_info(&tx, build_ctx_message).await;
        let build_context = helpers::create_build_context(all_files);

        // Build Docker image
        log::send_info(&tx, format!("Building image '{run_id}'...")).await;
        let image_labels = [
            ("tinirun", "v".to_owned() + env!("CARGO_PKG_VERSION")),
            (EXEC_LABEL, run_id.to_owned()),
        ];
        let build_stream = self.client.build_image(
            BuildImageOptionsBuilder::new()
                .t(&run_id)
                .labels(&image_labels.into())
                .build(),
            None,
            Some(bollard::body_try_stream(build_context)),
        );
        let (image_id, build_logs) = helpers::process_build_stream(build_stream, &tx).await;
        if let Some(image_id) = image_id {
            log::send_info(&tx, format!("Built image '{run_id}' with ID {image_id}")).await;
        } else {
            let _ = tx
                .send(CodeRunnerChunk::BuildError {
                    message: format!("Failed to build image '{run_id}'"),
                    build_logs,
                })
                .await;
            return;
        }

        // Create the container
        let (body, options) =
            helpers::setup_container(&run_id, &run_id, &command, timeout, mem_limit_mb, cpu_limit);
        if let Err(err) = self.client.create_container(Some(options), body).await {
            log::send_error(&tx, format!("Failed to create container '{run_id}': {err}")).await;
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
                let task = helpers::attach_task(attached, None, timeout, tx.clone());
                tokio::spawn(task)
            }
            Err(err) => {
                log::send_error(&tx, format!("Failed to attach to container: {err}")).await;
                return;
            }
        };

        // Start container
        log::send_info(&tx, format!("Starting container with '{command}'...")).await;
        if let Err(e) = self.client.start_container(&run_id, None).await {
            let message = format!("Failed to start container '{run_id}': {e}");
            log::send_error(&tx, message.clone()).await;
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
