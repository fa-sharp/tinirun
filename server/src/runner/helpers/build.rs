use std::path::PathBuf;

use bollard::models::BuildInfo;
use futures::{Stream, StreamExt};
use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;
use tokio_util::io::ReaderStream;

use crate::runner::helpers::log::{send_debug, send_info};

/// Create the build context as a tar archive to send to the Docker instance. Returns a ReaderStream
/// that can be passed to the Docker build API.
pub fn create_build_context(
    files: Vec<(PathBuf, Vec<u8>)>,
) -> ReaderStream<tokio::io::DuplexStream> {
    let (tar_writer, tar_reader) = tokio::io::duplex(8192); // 8KB max buffer
    tokio::spawn(async move {
        let mut tar = tokio_tar::Builder::new(tar_writer);
        for (path, content) in files {
            let mut header = tokio_tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            tar.append_data(&mut header, &path, content.as_slice())
                .await?;
        }
        tar.finish().await
    });

    ReaderStream::new(tar_reader)
}

/// Process the build stream from Docker and send logs to the client.
/// Returns the image ID if build was successful, along with build logs.
pub async fn process_build_stream(
    mut build_stream: impl Stream<Item = Result<BuildInfo, bollard::errors::Error>> + Unpin,
    tx: &mpsc::Sender<CodeRunnerChunk>,
) -> (Option<String>, String) {
    let mut image_id = None;
    let mut build_logs = String::with_capacity(1024);
    while let Some(build_info_result) = build_stream.next().await {
        match build_info_result {
            Ok(info) => {
                if let Some(id) = info.aux.and_then(|aux| aux.id) {
                    image_id = Some(id);
                }
                if let Some(stream) = info.stream {
                    build_logs.push_str(&stream);
                    send_debug(tx, stream).await;
                }
                if let Some(err) = info.error_detail.and_then(|e| e.message) {
                    let message = format!("Error during build: {err}");
                    build_logs.push('\n');
                    build_logs.push_str(&message);
                    send_info(tx, message).await;
                }
            }
            Err(err) => {
                let message = format!("Error during build: {err}");
                build_logs.push('\n');
                build_logs.push_str(&message);
                send_info(tx, message).await;
            }
        }
    }

    (image_id, build_logs)
}
