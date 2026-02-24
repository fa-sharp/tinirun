use std::time::Duration;

use bollard::{
    Docker,
    query_parameters::{PruneImagesOptionsBuilder, RemoveContainerOptionsBuilder},
};

use crate::runner::constants::EXEC_LABEL;

/// Cleanup Docker resources associated with a code execution run.
pub async fn run_cleanup(docker: &Docker, run_id: &str) {
    // Stop the container, ignoring errors in case it wasn't started or is already stopped
    let _ = docker.stop_container(run_id, None).await;

    // Remove the container
    let opt = RemoveContainerOptionsBuilder::new().force(true).build();
    if let Err(err) = docker.remove_container(run_id, Some(opt)).await {
        tracing::info!("Could not remove container '{run_id}': {err}");
    }
}

/// Task to periodically clean up Docker images created by code execution runs.
pub async fn image_cleanup_task(docker: Docker, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;

        // Clean up images created **before** the specified duration to avoid affecting current runs
        let until = format!("{}s", period.as_secs());
        let filters = [
            ("label", vec![EXEC_LABEL]),
            ("until", vec![&until]),
            ("dangling", vec!["false"]),
        ];
        let prune_options = PruneImagesOptionsBuilder::new()
            .filters(&filters.into())
            .build();
        match docker.prune_images(Some(prune_options)).await {
            Ok(res) => {
                if let Some(images) = res.images_deleted
                    && images.len() > 0
                {
                    let mb = res.space_reclaimed.unwrap_or_default() as f32 / 1024.0 / 1024.0;
                    tracing::info!("Pruned {} images, reclaimed {mb:.2} MB", images.len());
                }
            }
            Err(err) => tracing::warn!("Failed to prune images: {err}"),
        }
    }
}
