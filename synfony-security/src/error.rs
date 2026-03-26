use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Security-related errors.
///
/// Mirrors Symfony's AuthenticationException and AccessDeniedException.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication required")]
    Unauthenticated,

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Insufficient role: required {required}")]
    InsufficientRole { required: String },
}

#[derive(Serialize)]
struct AuthErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    title: String,
    status: u16,
    detail: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, title) = match &self {
            AuthError::Unauthenticated => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::InvalidCredentials(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::AccessDenied(_) => (StatusCode::FORBIDDEN, "Forbidden"),
            AuthError::InsufficientRole { .. } => (StatusCode::FORBIDDEN, "Forbidden"),
        };

        let body = AuthErrorResponse {
            error_type: format!("https://httpstatuses.com/{}", status.as_u16()),
            title: title.to_string(),
            status: status.as_u16(),
            detail: self.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}
