use axum_app_wrapper::AdHocPlugin;

use crate::{auth::ApiKey, state::AppState};

pub mod run_code;

/// Adds all API routes to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_setup(|router, state| {
        // Build API routes
        let api_router = aide::axum::ApiRouter::new()
            .route("/code/run", run_code::route())
            .layer(axum::middleware::from_extractor_with_state::<ApiKey, _>(
                state.clone(),
            ));

        // OpenAPI configuration
        let mut openapi = aide::openapi::OpenApi {
            info: aide::openapi::Info {
                title: "tinirun".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some("A simple code runner service using Docker".to_string()),
                ..Default::default()
            },
            servers: vec![aide::openapi::Server {
                url: "/api".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Add API routes to the router under `/api` and also merge them into the OpenAPI docs
        let router = router.nest("/api", api_router.finish_api(&mut openapi));

        // Add OpenAPI documentation routes
        let openapi_json = serde_json::to_string_pretty(&openapi).unwrap();
        let openapi_route = axum::routing::get(|| async move { openapi_json });
        let swagger_route = aide::swagger::Swagger::new("/api/openapi.json").axum_route();
        let router = router
            .route("/api/openapi.json", openapi_route)
            .route("/api/docs", swagger_route.into());

        Ok(router)
    })
}
