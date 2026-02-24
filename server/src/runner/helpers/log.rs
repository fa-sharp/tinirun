//! Logging utilities

use tinirun_models::CodeRunnerChunk;
use tokio::sync::mpsc;

pub async fn send_info(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Info(message)).await;
}
pub async fn send_debug(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Debug(message)).await;
}
pub async fn send_error(tx: &mpsc::Sender<CodeRunnerChunk>, message: String) {
    let _ = tx.send(CodeRunnerChunk::Error(message)).await;
}
