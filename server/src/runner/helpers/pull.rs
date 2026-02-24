use bollard::{Docker, models::CreateImageInfo, query_parameters::CreateImageOptionsBuilder};
use futures::{Stream, StreamExt};
use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;

use crate::runner::helpers::log;

// Check if a Docker image exists locally
pub async fn exists_image(client: &Docker, image: &str) -> Result<bool, bollard::errors::Error> {
    match client.inspect_image(image).await {
        Ok(_) => Ok(true),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(false),
        Err(err) => Err(err),
    }
}

/// Pull the given image if it doesn't exist and stream logs to the client.
pub async fn pull_image(
    client: &Docker,
    image: &str,
    tx: &mpsc::Sender<CodeRunnerChunk>,
) -> Result<(), bollard::errors::Error> {
    match exists_image(client, image).await? {
        true => Ok(()),
        false => {
            log::send_info(&tx, format!("Pulling base image '{image}'...")).await;
            let image_options = CreateImageOptionsBuilder::new().from_image(&image).build();
            let pull_stream = client.create_image(Some(image_options), None, None);
            process_pull_stream(pull_stream, &tx).await
        }
    }
}

/// Process the pull_image output from Docker and send logs to the client.
async fn process_pull_stream(
    mut pull_stream: impl Stream<Item = Result<CreateImageInfo, bollard::errors::Error>> + Unpin,
    tx: &mpsc::Sender<CodeRunnerChunk>,
) -> Result<(), bollard::errors::Error> {
    while let Some(result) = pull_stream.next().await {
        match result {
            Ok(mut info) => {
                let status = info.status.unwrap_or_default();
                let progress_detail = info.progress_detail.take().unwrap_or_default();
                if let Some((current, total)) = progress_detail.current.zip(progress_detail.total) {
                    log::send_debug(&tx, format!("Pulling image: {status} {current}/{total}"))
                        .await;
                }
                if let Some(error_detail) = info.error_detail {
                    log::send_info(&tx, format!("Error while pulling image: {error_detail:?}"))
                        .await;
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
