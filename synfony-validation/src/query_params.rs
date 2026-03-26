use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ValidationError;

/// Auto-validating query parameter extractor.
///
/// Equivalent to Symfony's `#[MapQueryString]`. Deserializes query parameters
/// into `T` and validates them.
///
/// # Example
/// ```ignore
/// #[derive(Deserialize, Validate)]
/// struct SearchParams {
///     #[validate(length(min = 1))]
///     q: String,
///     #[validate(range(min = 1, max = 100))]
///     limit: Option<u32>,
///     #[validate(range(min = 0))]
///     offset: Option<u32>,
/// }
///
/// #[route(GET, "/search")]
/// async fn search(params: QueryParams<SearchParams>) -> Json<Vec<Result>> {
///     let search = params.into_inner();
///     // search.q is guaranteed to be non-empty
/// }
/// ```
pub struct QueryParams<T>(pub T);

impl<T> QueryParams<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for QueryParams<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, S> FromRequestParts<S> for QueryParams<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) =
            axum::extract::Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|e| ValidationError::InvalidQuery(e.body_text()))?;

        value
            .validate()
            .map_err(ValidationError::ValidationFailed)?;

        Ok(QueryParams(value))
    }
}
