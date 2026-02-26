use std::{path::PathBuf, time::Duration};

use bollard::{
    Docker,
    query_parameters::{AttachContainerOptionsBuilder, BuildImageOptionsBuilder},
};
use futures::StreamExt;
use tinirun_models::{
    CodeRunnerChunk, CodeRunnerError, CodeRunnerFunctionResult, RunFunctionInput,
};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    redis::FunctionDetail,
    runner::{
        constants::{APP_LABEL, BUILD_ID_ARG, FN_LABEL},
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
    fn fn_tag(name: &str) -> String {
        format!("code-runner-fn-{name}:latest")
    }

    /// Build the function's Docker image. Returns the image tag and ID on success.
    pub async fn build_fn(
        &self,
        fn_name: &str,
        fn_info: FunctionDetail,
        lang_data: LanguageData,
        dockerfile: String,
        main_code: String,
        tx: mpsc::Sender<CodeRunnerChunk>,
    ) -> Result<(String, String), CodeRunnerError> {
        let image_tag = Self::fn_tag(&fn_name);
        let LanguageData {
            image: base_image,
            main_filename,
            fn_filename,
            ..
        } = lang_data;

        // Check if base image exists locally, and pull if needed
        log::send_info(&tx, format!("Checking base image '{base_image}'...")).await;
        if let Err(e) = helpers::pull_image(&self.client, &base_image, &tx).await {
            return Err(CodeRunnerError::Docker {
                message: format!("Error while checking/pulling base image: {e}"),
            });
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
            (APP_LABEL, "v".to_owned() + env!("CARGO_PKG_VERSION")),
            (FN_LABEL, fn_name.to_owned()),
        ];
        let build_stream = self.client.build_image(
            BuildImageOptionsBuilder::new()
                .t(&image_tag)
                .buildargs(&[(BUILD_ID_ARG, &format!("{fn_name}-v{}", fn_info.version))].into())
                .labels(&image_labels.into())
                .build(),
            None,
            Some(bollard::body_try_stream(build_context)),
        );

        match helpers::process_build_stream(build_stream, &tx).await {
            (Some(image_id), _) => {
                log::send_info(&tx, format!("Built image '{image_tag}' with ID {image_id}")).await;
                Ok((image_tag, image_id))
            }
            (None, logs) => Err(CodeRunnerError::BuildFailed {
                message: format!("Failed to build image '{image_tag}'"),
                logs,
            }),
        }
    }

    pub async fn run_function(
        &self,
        run_id: &str,
        fn_name: &str,
        input: RunFunctionInput,
        lang_data: LanguageData,
        tx: mpsc::Sender<CodeRunnerChunk>,
    ) -> Result<CodeRunnerFunctionResult, CodeRunnerError> {
        // Function input and language config
        let RunFunctionInput {
            input,
            timeout,
            mem_limit_mb,
            cpu_limit,
        } = input;
        let LanguageData { command, .. } = lang_data;
        let image_tag = Self::fn_tag(fn_name);

        // Ensure function image exists
        if !helpers::exists_image(&self.client, &image_tag).await? {
            return Err(CodeRunnerError::FunctionImageNotFound {
                message: format!("Image missing for '{fn_name}'. Please rebuild the function."),
                image_tag,
            });
        }

        // Create the container
        let (create_body, create_opt) = helpers::setup_container(
            &run_id,
            &image_tag,
            &command,
            true,
            timeout,
            mem_limit_mb,
            cpu_limit,
        );
        self.client
            .create_container(Some(create_opt), create_body)
            .await?;

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
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            timeout,
            exit_code,
        };
        tx.send(result_chunk).await.ok();

        Ok(CodeRunnerFunctionResult {
            input,
            stdout,
            stderr,
            exit_code,
            timeout,
        })
    }
}
