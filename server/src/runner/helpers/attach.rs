use std::time::Duration;

use bollard::container::LogOutput;
use futures::{Stream, StreamExt};
use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;

use crate::runner::helpers::log;

/// Maximum number of bytes accumulated for stdout or stderr.
/// Output beyond this limit is silently dropped to prevent memory exhaustion.
const MAX_OUTPUT_BYTES: usize = 1024 * 1024; // 1 MB
/// Grace period for the container to start
const GRACE_PERIOD_SECS: u32 = 5;

/// Attach to the Docker container output and send stdout/stderr logs back
/// to the client, while also returning the accumulated output at the
/// end of execution.
pub async fn output_task(
    mut output_stream: impl Stream<Item = Result<LogOutput, bollard::errors::Error>> + Unpin,
    timeout: u32,
    tx: mpsc::Sender<CodeRunnerChunk>,
) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    let timeout_duration = Duration::from_secs((timeout + GRACE_PERIOD_SECS).into());

    let _ = tokio::time::timeout(timeout_duration, async {
        while let Some(output_result) = output_stream.next().await {
            match output_result {
                Ok(output) => match output {
                    LogOutput::StdOut { message } => {
                        if let Ok(message_str) = String::from_utf8(message.into()) {
                            if stdout.len() < MAX_OUTPUT_BYTES {
                                stdout.push_str(&message_str);
                                stdout.push('\n');
                            }
                            tx.send(CodeRunnerChunk::Stdout(message_str)).await.ok();
                        }
                    }
                    LogOutput::StdErr { message } => {
                        if let Ok(message_str) = String::from_utf8(message.into()) {
                            if stderr.len() < MAX_OUTPUT_BYTES {
                                stderr.push_str(&message_str);
                                stderr.push('\n');
                            }
                            tx.send(CodeRunnerChunk::Stderr(message_str)).await.ok();
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    let message = format!("Error while processing output: {e}");
                    log::send_info(&tx, message).await;
                }
            }
        }
    })
    .await;

    (stdout, stderr)
}
