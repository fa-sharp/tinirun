use aide::OperationOutput;
use axum::{
    Json,
    response::{IntoResponse, Sse, sse::Event},
};
use axum_streams::StreamBodyAs;
use futures::{Stream, StreamExt};
use schemars::JsonSchema;
use serde::Serialize;

use crate::input::StreamType;

/// Represents a stream response with a specific chunk type that will be documented
/// in the OpenAPI specification.
pub struct StreamResponse<S, Chunk>
where
    S: Stream<Item = Chunk> + Send + 'static,
{
    stream: S,
    stream_type: StreamType,
}

impl<S, Chunk> StreamResponse<S, Chunk>
where
    S: Stream<Item = Chunk> + Send + 'static,
{
    pub fn new(stream: S, stream_type: StreamType) -> Self {
        Self {
            stream,
            stream_type,
        }
    }
}

impl<S, Chunk> IntoResponse for StreamResponse<S, Chunk>
where
    S: Stream<Item = Chunk> + Send + 'static,
    Chunk: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> axum::response::Response {
        match self.stream_type {
            StreamType::Sse => {
                Sse::new(self.stream.map(|chunk| Event::default().json_data(chunk))).into_response()
            }
            StreamType::Jsonl => StreamBodyAs::json_nl(self.stream).into_response(),
        }
    }
}

impl<S, Chunk> OperationOutput for StreamResponse<S, Chunk>
where
    S: Stream<Item = Chunk> + Send + 'static,
    Chunk: Serialize + JsonSchema,
{
    type Inner = Chunk;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        if let Some(mut operation_response) = Json::<Chunk>::operation_response(ctx, operation) {
            let schema_object = aide::openapi::SchemaObject {
                json_schema: ctx.schema.subschema_for::<Chunk>(),
                example: None,
                external_docs: None,
            };
            ["text/event-stream", "application/jsonl"]
                .into_iter()
                .for_each(|mime| {
                    operation_response.content.insert(
                        mime.into(),
                        aide::openapi::MediaType {
                            schema: Some(schema_object.clone()),
                            ..Default::default()
                        },
                    );
                });
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
