use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Structured API error that maps to HTTP status codes.
///
/// Equivalent to Symfony's HttpException hierarchy. Returns JSON error
/// responses automatically when returned from handlers.
///
/// # Example
/// ```ignore
/// #[route(GET, "/:id")]
/// async fn show(Path(id): Path<i32>) -> Result<Json<User>, ApiError> {
///     let user = repo.find(id).await?
///         .ok_or(ApiError::not_found("User not found"))?;
///     Ok(Json(user))
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{message}")]
    BadRequest { message: String },

    #[error("{message}")]
    Unauthorized { message: String },

    #[error("{message}")]
    Forbidden { message: String },

    #[error("{message}")]
    NotFound { message: String },

    #[error("{message}")]
    Conflict { message: String },

    #[error("{message}")]
    UnprocessableEntity {
        message: String,
        errors: Option<serde_json::Value>,
    },

    #[error("{message}")]
    Internal { message: String },
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest {
            message: msg.into(),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ApiError::Unauthorized {
            message: msg.into(),
        }
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        ApiError::Forbidden {
            message: msg.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound {
            message: msg.into(),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        ApiError::Conflict {
            message: msg.into(),
        }
    }

    pub fn unprocessable(msg: impl Into<String>, errors: Option<serde_json::Value>) -> Self {
        ApiError::UnprocessableEntity {
            message: msg.into(),
            errors,
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal {
            message: msg.into(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
            ApiError::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// JSON error response body matching Symfony's standard error format.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error_type: format!("https://httpstatuses.com/{}", status.as_u16()),
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: self.to_string(),
            errors: match &self {
                ApiError::UnprocessableEntity { errors, .. } => errors.clone(),
                _ => None,
            },
        };

        (status, axum::Json(body)).into_response()
    }
}

// Allow using anyhow errors as internal server errors
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Internal error: {:?}", err);
        ApiError::Internal {
            message: "An internal error occurred".to_string(),
        }
    }
}
