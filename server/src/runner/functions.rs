use std::{path::PathBuf, time::Duration};

use bollard::{
    Docker,
    query_parameters::{AttachContainerOptionsBuilder, BuildImageOptionsBuilder},
};
use futures::StreamExt;
use tinirun_models::{CodeRunnerChunk, RunFunctionInput};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    redis::{FunctionDetail, FunctionInfo},
    runner::{
        constants::FN_LABEL,
        helpers::{self, log},
        structs::LanguageData,
    },
};

pub struct FunctionExecutor {
    pub client: Docker,
}

impl FunctionExecutor {
    pub fn new(client: Docker) -> Self {
        Self { client }
    }

    /// The tag of the function's Docker image
    fn fn_tag(name: &str, version: u32) -> String {
        format!("code-runner-fn-{name}-v{version}")
    }

    pub async fn build_fn(
        &self,
        fn_name: &str,
        fn_info: FunctionDetail,
        lang_data: LanguageData,
        dockerfile: String,
        main_code: String,
        tx: mpsc::Sender<CodeRunnerChunk>,
    ) -> Result<(), ()> {
        let image_tag = Self::fn_tag(&fn_name, fn_info.version);
        let LanguageData {
            image: base_image,
            main_filename,
            fn_filename,
            ..
        } = lang_data;

        // Check if base image exists locally, and pull if needed
        log::send_info(&tx, format!("Checking base image '{base_image}'...")).await;
        if let Err(e) = helpers::pull_image(&self.client, &base_image, &tx).await {
            log::send_error(&tx, format!("Error while checking/pulling image: {e}")).await;
            return Err(());
        }

        // Create build context (Dockerfile, main code, function code)
        let all_files = vec![
            (PathBuf::from("Dockerfile"), dockerfile.into_bytes()),
            (PathBuf::from(main_filename), main_code.into_bytes()),
            (PathBuf::from(fn_filename), fn_info.code.into_bytes()),
        ];
        let mut build_ctx_message = String::from("Creating build context:");
        for (path, _) in all_files.iter() {
            build_ctx_message.push_str(&format!(" {path:?}"));
        }
        log::send_info(&tx, build_ctx_message).await;
        let build_context = helpers::create_build_context(all_files);

        // Build Docker image
        log::send_info(&tx, format!("Building image '{image_tag}'...")).await;
        let image_labels = [
            ("tinirun", "v".to_owned() + env!("CARGO_PKG_VERSION")),
            (FN_LABEL, fn_name.to_owned()),
        ];
        let build_stream = self.client.build_image(
            BuildImageOptionsBuilder::new()
                .t(&image_tag)
                .labels(&image_labels.into())
                .build(),
            None,
            Some(bollard::body_try_stream(build_context)),
        );

        match helpers::process_build_stream(build_stream, &tx).await {
            (Some(image_id), _) => {
                log::send_info(&tx, format!("Built image '{image_tag}' with ID {image_id}")).await;
                Ok(())
            }
            (None, build_logs) => {
                let message = CodeRunnerChunk::BuildError {
                    message: format!("Failed to build image '{image_tag}'"),
                    build_logs,
                };
                tx.send(message).await.ok();
                Err(())
            }
        }
    }

    pub async fn run_function(
        &self,
        run_id: &str,
        fn_name: &str,
        fn_info: FunctionInfo,
        input: RunFunctionInput,
        lang_data: LanguageData,
        tx: mpsc::Sender<CodeRunnerChunk>,
    ) -> Result<CodeRunnerChunk, bollard::errors::Error> {
        // Function input and language config
        let RunFunctionInput {
            input,
            timeout,
            mem_limit_mb,
            cpu_limit,
        } = input;
        let LanguageData { command, .. } = lang_data;
        let image_tag = Self::fn_tag(fn_name, fn_info.version);

        // Ensure function image exists
        if !helpers::exists_image(&self.client, &image_tag).await? {
            log::send_error(&tx, "No image - function may need to be rebuilt.".into()).await;
            return Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404,
                message: "Image not found".into(),
            });
        }

        // Create the container
        let (body, options) = helpers::setup_container(
            &run_id,
            &image_tag,
            &command,
            true,
            timeout,
            mem_limit_mb,
            cpu_limit,
        );
        self.client.create_container(Some(options), body).await?;

        // Attach to container and setup capturing of logs/output
        let attach_options = AttachContainerOptionsBuilder::new()
            .stream(true)
            .stdin(true)
            .stdout(true)
            .stderr(true)
            .logs(true)
            .build();
        let mut container = self
            .client
            .attach_container(&run_id, Some(attach_options.clone()))
            .await?;
        let container_output =
            tokio::spawn(helpers::output_task(container.output, timeout, tx.clone()));

        // Start container and write input to stdin
        log::send_info(&tx, format!("Starting container with '{command}'...")).await;
        self.client.start_container(&run_id, None).await?;

        log::send_info(&tx, "Writing input to container".into()).await;
        if let Err(err) = container.input.write_all(input.as_bytes()).await {
            log::send_info(&tx, format!("Failed to write input: {err}")).await;
        }
        if let Err(err) = container.input.shutdown().await {
            log::send_info(&tx, format!("Failed to flush input: {err}")).await;
        }
        drop(container.input);

        // Wait for container to exit, then get exit status and final stdout and stderr
        let container_exit_result = tokio::time::timeout(
            Duration::from_secs(timeout.into()),
            self.client.wait_container(&run_id, None).next(),
        )
        .await;
        let (stdout, stderr) = container_output.await.unwrap_or_default();
        let (timeout, exit_code) = helpers::process_exit_status(container_exit_result);
        let result_chunk = CodeRunnerChunk::Result {
            stdout,
            stderr,
            timeout,
            exit_code,
        };
        tx.send(result_chunk.clone()).await.ok();

        Ok(result_chunk)
    }
}
