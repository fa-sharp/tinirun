use bollard::models::BuildInfo;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_util::io::ReaderStream;

use crate::runner::{
    executor::{send_debug, send_error},
    structs::CodeRunnerChunk,
};

/// Create the build context as a tar archive to send to the Docker instance. Returns a ReaderStream
/// that can be passed to the Docker build API.
pub async fn create_build_context(
    code: String,
    main_file: String,
    dockerfile: String,
) -> ReaderStream<tokio::io::DuplexStream> {
    let (tar_writer, tar_reader) = tokio::io::duplex(8192); // 8KB max buffer
    tokio::spawn(async move {
        let mut tar = tokio_tar::Builder::new(tar_writer);
        for (path, content) in [("Dockerfile", dockerfile), (&main_file, code)] {
            let mut header = tokio_tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            tar.append_data(&mut header, path, content.as_bytes())
                .await?;
        }
        tar.finish().await
    });

    ReaderStream::new(tar_reader)
}

/// Process the build stream from Docker and send logs to the client.
pub async fn process_build_stream(
    mut build_stream: impl Stream<Item = Result<BuildInfo, bollard::errors::Error>> + Unpin,
    tx: &mpsc::Sender<CodeRunnerChunk>,
) -> Option<String> {
    let mut image_id = None;
    while let Some(build_info_result) = build_stream.next().await {
        match build_info_result {
            Ok(info) => {
                if let Some(id) = info.aux.and_then(|aux| aux.id) {
                    image_id = Some(id);
                }
                if let Some(stream) = info.stream {
                    send_debug(tx, stream).await;
                }
                if let Some(err) = info.error_detail.and_then(|e| e.message) {
                    send_error(tx, format!("Error during build: {err}")).await;
                }
            }
            Err(err) => send_error(tx, format!("Error during build: {err}")).await,
        }
    }

    image_id
}
