use bollard::{Docker, query_parameters::RemoveContainerOptionsBuilder};

/// Cleanup Docker resources associated with a code execution run.
pub async fn docker_cleanup(docker: &Docker, run_id: &str) {
    // Stop the container, ignoring errors in case it wasn't started or is already stopped
    let _ = docker.stop_container(run_id, None).await;

    // Remove the container
    let opt = RemoveContainerOptionsBuilder::new().force(true).build();
    if let Err(err) = docker.remove_container(run_id, Some(opt)).await {
        tracing::info!("Could not remove container '{run_id}': {err}");
    }
}
