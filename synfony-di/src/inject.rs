use std::ops::Deref;
use std::sync::Arc;

/// A dependency injection wrapper that resolves services from the container.
///
/// `Inject<T>` is the Synfony equivalent of Symfony's autowired constructor parameters.
/// It wraps an `Arc<T>` and implements Axum's `FromRequestParts` so it can be used
/// directly in handler function signatures.
///
/// # Example
/// ```ignore
/// #[route(GET, "/users")]
/// async fn list(repo: Inject<UserRepository>) -> Json<Vec<User>> {
///     let users = repo.find_all().await?;
///     Json(users)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Inject<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized> Inject<T> {
    pub fn new(inner: Arc<T>) -> Self {
        Inject(inner)
    }
}

impl<T: ?Sized> Deref for Inject<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> AsRef<T> for Inject<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> From<Arc<T>> for Inject<T> {
    fn from(arc: Arc<T>) -> Self {
        Inject(arc)
    }
}

// Axum extractor implementation — resolves T from the AppState's container.
// In axum 0.8+, FromRequestParts is a regular trait (not async_trait).
impl<T, S> axum::extract::FromRequestParts<S> for Inject<T>
where
    T: 'static + Send + Sync,
    S: Send + Sync + AsRef<crate::Container>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let container: &crate::Container = state.as_ref();
        Ok(Inject(container.resolve::<T>()))
    }
}
