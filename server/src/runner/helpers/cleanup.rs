use std::time::Duration;

use bollard::{
    Docker,
    query_parameters::{
        PruneContainersOptionsBuilder, PruneImagesOptionsBuilder, RemoveContainerOptionsBuilder,
    },
};

use crate::runner::constants::{APP_LABEL, EXEC_LABEL};

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

/// Task to periodically clean up Docker images and containers created by code execution runs.
pub async fn image_cleanup_task(docker: Docker, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;

        // Clean up resources created **before** the specified duration
        let until = format!("{}s", period.as_secs());

        // Prune any stopped containers that are lying around
        let prune_container_opt = PruneContainersOptionsBuilder::new()
            .filters(&[("label", vec![APP_LABEL]), ("until", vec![&until])].into())
            .build();
        match docker.prune_containers(Some(prune_container_opt)).await {
            Ok(res) => {
                if let Some(containers) = res.containers_deleted
                    && containers.len() > 0
                {
                    let mb = res.space_reclaimed.unwrap_or_default() as f32 / 1024.0 / 1024.0;
                    tracing::info!("Pruned {} containers, saved {mb:.2} MB", containers.len());
                }
            }
            Err(err) => tracing::warn!("Failed to prune containers: {err}"),
        }

        // Prune one-off code execution images
        let image_filters = [
            ("label", vec![EXEC_LABEL]),
            ("until", vec![&until]),
            ("dangling", vec!["false"]),
        ];
        let prune_image_opt = PruneImagesOptionsBuilder::new()
            .filters(&image_filters.into())
            .build();
        match docker.prune_images(Some(prune_image_opt)).await {
            Ok(res) => {
                if let Some(images) = res.images_deleted
                    && images.len() > 0
                {
                    let mb = res.space_reclaimed.unwrap_or_default() as f32 / 1024.0 / 1024.0;
                    tracing::info!("Pruned {} images, saved {mb:.2} MB", images.len());
                }
            }
            Err(err) => tracing::warn!("Failed to prune images: {err}"),
        }
    }
}
