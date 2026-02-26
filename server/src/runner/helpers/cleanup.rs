use std::time::Duration;

use bollard::{
    Docker,
    query_parameters::{
        ListImagesOptionsBuilder, PruneContainersOptionsBuilder, PruneImagesOptionsBuilder,
        RemoveContainerOptionsBuilder, RemoveImageOptions,
    },
};

use crate::runner::constants::{APP_LABEL, EXEC_LABEL, FN_LABEL};

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

        // Keep track of number of images pruned
        let mut num_pruned = 0;

        // Prune one-off code execution images
        let prune_filters = [
            ("label", vec![EXEC_LABEL]),
            ("until", vec![&until]),
            ("dangling", vec!["false"]),
        ];
        let prune_image_opt = PruneImagesOptionsBuilder::new()
            .filters(&prune_filters.into())
            .build();
        match docker.prune_images(Some(prune_image_opt)).await {
            Ok(res) => {
                if let Some(images) = res.images_deleted {
                    num_pruned += images.len();
                }
            }
            Err(err) => tracing::warn!("Failed to prune execution images: {err}"),
        }

        // Prune old function images (any not tagged as 'latest')
        let list_image_opt = ListImagesOptionsBuilder::new()
            .filters(&[("label", vec![FN_LABEL]), ("until", vec![&until])].into())
            .build();
        let Ok(fn_images) = docker.list_images(Some(list_image_opt)).await else {
            tracing::warn!("Failed to list function images");
            continue;
        };
        for old_image in fn_images
            .into_iter()
            .filter(|image| image.repo_tags.iter().all(|tag| !tag.ends_with(":latest")))
        {
            match docker
                .remove_image(&old_image.id, None::<RemoveImageOptions>, None)
                .await
            {
                Ok(deleted) => num_pruned += deleted.len(),
                Err(err) => tracing::warn!("Failed to prune old function image: {err}"),
            }
        }

        if num_pruned > 0 {
            tracing::info!("Pruned {num_pruned} images");
        }
    }
}
