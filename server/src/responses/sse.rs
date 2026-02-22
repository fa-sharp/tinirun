use std::marker::PhantomData;

use aide::OperationOutput;
use axum::{
    Json,
    response::{IntoResponse, Sse, sse::Event},
};
use futures::Stream;
use schemars::JsonSchema;

/// Represents an SSE response with a specific chunk type that will be
/// documented in the OpenAPI specification.
pub struct SseResponse<S, Chunk> {
    sse: Sse<S>,
    chunk: PhantomData<Chunk>,
}

impl<S, Chunk> SseResponse<S, Chunk>
where
    S: Stream<Item = Result<Event, axum::Error>> + Send + 'static,
{
    pub fn new(stream: S) -> Self {
        Self {
            sse: Sse::new(stream),
            chunk: PhantomData,
        }
    }
}

impl<S, Chunk> IntoResponse for SseResponse<S, Chunk>
where
    Chunk: JsonSchema,
    S: Stream<Item = Result<Event, axum::Error>> + Send + 'static,
{
    fn into_response(self) -> axum::response::Response {
        self.sse.into_response()
    }
}

impl<S, Chunk> OperationOutput for SseResponse<S, Chunk>
where
    Chunk: JsonSchema,
{
    type Inner = Chunk;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        if let Some(mut operation_response) = Json::<Chunk>::operation_response(ctx, operation) {
            operation_response.content = FromIterator::from_iter([(
                "text/event-stream".into(),
                aide::openapi::MediaType {
                    schema: Some(aide::openapi::SchemaObject {
                        json_schema: ctx.schema.subschema_for::<Chunk>(),
                        example: None,
                        external_docs: None,
                    }),
                    ..Default::default()
                },
            )]);
            Some(operation_response)
        } else {
            None
        }
    }

    fn inferred_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        if let Some(res) = Self::operation_response(ctx, operation) {
            vec![(Some(aide::openapi::StatusCode::Code(200)), res)]
        } else {
            vec![]
        }
    }
}
