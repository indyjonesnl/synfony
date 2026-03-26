use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AuthError;
use crate::token::SecurityToken;

/// Axum extractor for the currently authenticated user.
///
/// Equivalent to Symfony's `$this->getUser()` in controllers.
/// Extracts the `SecurityToken` from request extensions (set by the firewall middleware).
///
/// Returns `AuthError::Unauthenticated` if no token is present.
///
/// # Example
/// ```ignore
/// #[route(GET, "/me")]
/// async fn me(user: CurrentUser) -> Json<UserProfile> {
///     Json(UserProfile {
///         id: user.user_id().to_string(),
///         email: user.user_identifier().to_string(),
///         roles: user.roles().iter().cloned().collect(),
///     })
/// }
/// ```
pub struct CurrentUser(pub SecurityToken);

impl CurrentUser {
    pub fn token(&self) -> &SecurityToken {
        &self.0
    }

    pub fn user_id(&self) -> &str {
        self.0.user_id()
    }

    pub fn user_identifier(&self) -> &str {
        self.0.user_identifier()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.0.has_role(role)
    }

    pub fn roles(&self) -> &std::collections::HashSet<String> {
        self.0.roles()
    }
}

impl std::ops::Deref for CurrentUser {
    type Target = SecurityToken;
    fn deref(&self) -> &SecurityToken {
        &self.0
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SecurityToken>()
            .cloned()
            .map(CurrentUser)
            .ok_or(AuthError::Unauthenticated)
    }
}

/// Optional variant — returns None instead of an error when not authenticated.
///
/// Useful for endpoints that behave differently for logged-in vs anonymous users.
///
/// # Example
/// ```ignore
/// async fn index(user: OptionalUser) -> Json<Greeting> {
///     match user.0 {
///         Some(u) => Json(Greeting { msg: format!("Hello, {}!", u.user_identifier()) }),
///         None => Json(Greeting { msg: "Hello, guest!".into() }),
///     }
/// }
/// ```
pub struct OptionalUser(pub Option<SecurityToken>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalUser(parts.extensions.get::<SecurityToken>().cloned()))
    }
}
