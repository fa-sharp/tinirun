use aide::{OperationInput, openapi};
use axum::{Json, RequestExt, extract::FromRequest};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::input::InputValidationError;

/// JSON body extractor with validator support and JSON error messages
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
{
    type Rejection = InputValidationError;

    async fn from_request(
        req: axum::extract::Request,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match req.extract::<Json<T>, _>().await {
            Ok(Json(val)) => match val.validate() {
                Ok(_) => Ok(Self(val)),
                Err(err) => Err(InputValidationError::new(err.to_string())),
            },
            Err(err) => Err(InputValidationError::new(err.body_text())),
        }
    }
}

impl<T> OperationInput for AppJson<T>
where
    T: JsonSchema,
{
    fn operation_input(ctx: &mut aide::generate::GenContext, operation: &mut openapi::Operation) {
        Json::<T>::operation_input(ctx, operation);
    }
}
