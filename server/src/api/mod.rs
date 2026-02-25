use axum_app_wrapper::AdHocPlugin;
use strum::{Display, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

use crate::{
    auth::{API_KEY_HEADER, ApiKey},
    state::AppState,
};

pub mod function;
pub mod run_code;

/// Tags in the OpenAPI specification
#[derive(Debug, IntoStaticStr, Display, EnumMessage, EnumIter)]
enum ApiTag {
    #[strum(message = "Run code")]
    Run,
    #[strum(message = "Run and manage functions")]
    Functions,
}

/// Adds all API routes to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::new().on_setup(|router, state| {
        // Build API routes
        let api_router = aide::axum::ApiRouter::new()
            .api_route("/code/run", run_code::route())
            .nest("/function", function::routes())
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
            components: Some(aide::openapi::Components {
                security_schemes: FromIterator::from_iter([(
                    "ApiKey".to_owned(),
                    aide::openapi::ReferenceOr::Item(aide::openapi::SecurityScheme::ApiKey {
                        name: API_KEY_HEADER.to_owned(),
                        location: aide::openapi::ApiKeyLocation::Header,
                        description: Some("API key for authentication".to_string()),
                        extensions: Default::default(),
                    }),
                )]),
                ..Default::default()
            }),
            security: vec![[("ApiKey".to_owned(), vec!["ApiKey".to_owned()])].into()],
            tags: ApiTag::iter()
                .map(|tag| aide::openapi::Tag {
                    name: tag.to_string(),
                    description: tag.get_message().map(str::to_owned),
                    ..Default::default()
                })
                .collect(),
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
