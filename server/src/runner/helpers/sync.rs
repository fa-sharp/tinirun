use std::time::Duration;

use bollard::{Docker, query_parameters::ListImagesOptionsBuilder};

use crate::{
    redis::{FunctionStatus, RedisClient},
    runner::constants::FN_LABEL,
};

/// Task to sync the status of function images
pub async fn sync_function_status_task(docker: Docker, redis: RedisClient, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;

        // List functions and images
        let functions = match redis.list_functions(100).await {
            Ok(functions) => functions,
            Err(err) => {
                tracing::warn!("Failed to list functions in Redis: {err}");
                continue;
            }
        };
        let list_image_opt = ListImagesOptionsBuilder::new()
            .filters(&[("label", vec![FN_LABEL])].into())
            .build();
        let function_images = match docker.list_images(Some(list_image_opt)).await {
            Ok(images) => images,
            Err(err) => {
                tracing::warn!("Failed to list function images in Docker: {err}");
                continue;
            }
        };

        // If a function is marked ready, verify that the image exists and update the status if needed.
        for (fn_name, fn_info) in functions {
            match fn_info.status {
                FunctionStatus::Ready { id, .. } => {
                    if !function_images.iter().any(|image| image.id == id) {
                        let _ = redis
                            .set_fn_status(&fn_name, FunctionStatus::NotBuilt)
                            .await;
                    }
                }
                _ => {}
            }
        }
    }
}
