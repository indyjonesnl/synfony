use axum::http::request::Parts;
use axum::response::Response;
use std::sync::Arc;

use crate::error::AuthError;
use crate::token::SecurityToken;

/// The Authenticator trait — equivalent to Symfony's `AuthenticatorInterface`.
///
/// Authenticators extract credentials from the request and validate them.
/// Each firewall can have one authenticator assigned.
///
/// # Lifecycle
/// 1. `supports()` — Does this authenticator handle this request?
/// 2. `authenticate()` — Extract and validate credentials, return a SecurityToken
/// 3. `on_success()` — Optional: modify the response after successful auth
/// 4. `on_failure()` — Generate an error response on auth failure
///
/// # Example
/// ```ignore
/// struct ApiKeyAuthenticator {
///     valid_keys: HashSet<String>,
/// }
///
/// #[async_trait]
/// impl Authenticator for ApiKeyAuthenticator {
///     fn supports(&self, parts: &Parts) -> bool {
///         parts.headers.contains_key("X-API-Key")
///     }
///
///     async fn authenticate(&self, parts: &Parts) -> Result<SecurityToken, AuthError> {
///         let key = parts.headers.get("X-API-Key")
///             .and_then(|v| v.to_str().ok())
///             .ok_or(AuthError::Unauthenticated)?;
///         if self.valid_keys.contains(key) {
///             Ok(SecurityToken::new("api", key))
///         } else {
///             Err(AuthError::InvalidCredentials("Invalid API key".into()))
///         }
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Authenticator: Send + Sync {
    /// Does this authenticator support the given request?
    /// Return false to skip this authenticator (the firewall will reject unauthenticated requests).
    fn supports(&self, parts: &Parts) -> bool;

    /// Authenticate the request — extract credentials and verify them.
    /// Return a SecurityToken on success, AuthError on failure.
    async fn authenticate(&self, parts: &Parts) -> Result<SecurityToken, AuthError>;

    /// Called after successful authentication.
    /// Return None to continue to the handler, or Some(Response) to short-circuit.
    async fn on_success(
        &self,
        _parts: &Parts,
        _token: &SecurityToken,
    ) -> Option<Response> {
        None
    }

    /// Generate an error response for authentication failure.
    /// Default implementation returns the AuthError as a JSON response.
    fn on_failure(&self, error: AuthError) -> Response {
        axum::response::IntoResponse::into_response(error)
    }
}

/// Type-erased authenticator for storage in collections.
pub type AuthenticatorBox = Arc<dyn Authenticator>;
