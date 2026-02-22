use std::time::Duration;

use bollard::container::{AttachContainerResults, LogOutput};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::runner::structs::CodeRunnerChunk;

/// Maximum number of bytes accumulated for stdout or stderr.
/// Output beyond this limit is silently dropped to prevent memory exhaustion.
const MAX_OUTPUT_BYTES: usize = 1024 * 1024; // 1 MB

/// Attach to the Docker container and send stdout/stderr logs back to the client, while also
/// returning the accumulated output at the end of execution.
pub async fn attach_and_process_output(
    mut attached_container: AttachContainerResults,
    timeout: u32,
    tx: mpsc::Sender<CodeRunnerChunk>,
) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    let _ = tokio::time::timeout(Duration::from_secs(timeout.into()), async {
        while let Some(output_result) = attached_container.output.next().await {
            match output_result {
                Ok(output) => match output {
                    LogOutput::StdOut { message } => {
                        let message_str = String::from_utf8_lossy(&message).into_owned();
                        if stdout.len() < MAX_OUTPUT_BYTES {
                            stdout.push_str(&format!("{message_str}\n"));
                        }
                        tx.send(CodeRunnerChunk::Stdout(message_str)).await.ok();
                    }
                    LogOutput::StdErr { message } => {
                        let message_str = String::from_utf8_lossy(&message).into_owned();
                        if stderr.len() < MAX_OUTPUT_BYTES {
                            stderr.push_str(&format!("{message_str}\n"));
                        }
                        tx.send(CodeRunnerChunk::Stderr(message_str)).await.ok();
                    }
                    _ => {}
                },
                Err(e) => super::send_error(&tx, e.to_string()).await,
            }
        }
    })
    .await;

    (stdout, stderr)
}
