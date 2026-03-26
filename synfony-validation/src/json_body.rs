use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ValidationError;

/// Auto-validating JSON body extractor.
///
/// Equivalent to Symfony's `#[MapRequestPayload]`. Deserializes the JSON
/// request body into `T` and then validates it using the `validator` crate.
///
/// Returns 422 Unprocessable Entity with structured errors on failure.
///
/// # Example
/// ```ignore
/// #[derive(Deserialize, Validate)]
/// struct CreateUserDto {
///     #[validate(length(min = 2, max = 100))]
///     name: String,
///     #[validate(email)]
///     email: String,
/// }
///
/// #[route(POST, "/users")]
/// async fn create(payload: JsonBody<CreateUserDto>) -> Json<User> {
///     // payload.0 is guaranteed to be valid here
///     let dto = payload.into_inner();
///     // ...
/// }
/// ```
pub struct JsonBody<T>(pub T);

impl<T> JsonBody<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for JsonBody<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // First, deserialize the JSON body
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ValidationError::InvalidJson(e.body_text()))?;

        // Then validate
        value
            .validate()
            .map_err(ValidationError::ValidationFailed)?;

        Ok(JsonBody(value))
    }
}
