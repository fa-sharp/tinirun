//! Executor for running code in Docker containers

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use bollard::{
    Docker,
    query_parameters::{AttachContainerOptionsBuilder, BuildImageOptionsBuilder},
};
use futures::StreamExt;
use tinirun_models::{CodeRunnerChunk, CodeRunnerError};
use tokio::sync::mpsc;

use crate::runner::{
    constants::{APP_LABEL, BUILD_ID_ARG, EXEC_LABEL},
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
    ) -> Result<(), CodeRunnerError> {
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
            let message = format!("Error while checking/pulling image: {e}");
            return Err(CodeRunnerError::Docker { message });
        }

        // Create build context (Dockerfile, code, attached files)
        let code_files = [
            (PathBuf::from("Dockerfile"), dockerfile.into_bytes()),
            (PathBuf::from(main_file), code.into_bytes()),
        ];
        let attached_files = files
            .unwrap_or_default()
            .into_iter()
            .map(|file| (Path::new("files").join(file.path), file.content));
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
            (APP_LABEL, "v".to_owned() + env!("CARGO_PKG_VERSION")),
            (EXEC_LABEL, run_id.to_owned()),
        ];
        let build_stream = self.client.build_image(
            BuildImageOptionsBuilder::new()
                .t(&run_id)
                .buildargs(&[(BUILD_ID_ARG, run_id)].into())
                .labels(&image_labels.into())
                .build(),
            None,
            Some(bollard::body_try_stream(build_context)),
        );
        let (image_id, build_logs) = helpers::process_build_stream(build_stream, &tx).await;
        if let Some(image_id) = image_id {
            log::send_info(&tx, format!("Built image '{run_id}' with ID {image_id}")).await;
        } else {
            let err = CodeRunnerError::BuildFailed {
                message: format!("Failed to build image '{run_id}'"),
                logs: build_logs,
            };
            return Err(err);
        }

        // Create the container
        let (body, options) = helpers::setup_container(
            &run_id,
            &run_id,
            &command,
            false,
            timeout,
            mem_limit_mb,
            cpu_limit,
        );
        self.client.create_container(Some(options), body).await?;

        // Attach to container and setup capturing of logs/output
        let attach_options = AttachContainerOptionsBuilder::new()
            .stream(true)
            .stdout(true)
            .stderr(true)
            .logs(true)
            .build();
        let container = self
            .client
            .attach_container(&run_id, Some(attach_options))
            .await?;
        let capture_output = helpers::output_task(container.output, timeout, tx.clone());
        let output_task = tokio::spawn(capture_output);

        // Start container
        log::send_info(&tx, format!("Starting container with '{command}'...")).await;
        self.client.start_container(&run_id, None).await?;

        // Wait for container to exit, then get exit status and final stdout and stderr
        let exit_result = tokio::time::timeout(
            Duration::from_secs(timeout.into()),
            self.client.wait_container(&run_id, None).next(),
        )
        .await;
        let (stdout, stderr) = output_task.await.unwrap_or_default();
        let (timeout, exit_code) = helpers::process_exit_status(exit_result);

        let result_chunk = CodeRunnerChunk::Result {
            stdout,
            stderr,
            timeout,
            exit_code,
        };
        let _ = tx.send(result_chunk).await;

        Ok(())
    }
}
