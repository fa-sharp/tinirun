use bollard::models::CreateImageInfo;
use futures::{Stream, StreamExt};
use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;

use crate::runner::executor::{send_debug, send_info};

/// Process the pull_image output from Docker and send logs to the client.
pub async fn process_pull_stream(
    mut pull_stream: impl Stream<Item = Result<CreateImageInfo, bollard::errors::Error>> + Unpin,
    tx: &mpsc::Sender<CodeRunnerChunk>,
) -> Result<(), bollard::errors::Error> {
    while let Some(result) = pull_stream.next().await {
        match result {
            Ok(mut info) => {
                let status = info.status.unwrap_or_default();
                let progress_detail = info.progress_detail.take().unwrap_or_default();
                if let Some((current, total)) = progress_detail.current.zip(progress_detail.total) {
                    send_debug(&tx, format!("Pulling image: {status} {current}/{total}")).await;
                }
                if let Some(error_detail) = info.error_detail {
                    send_info(&tx, format!("Error while pulling image: {error_detail:?}")).await;
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
