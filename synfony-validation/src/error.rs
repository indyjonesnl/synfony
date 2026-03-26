use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use validator::ValidationErrors;

/// Structured validation error response.
///
/// Returns 422 Unprocessable Entity with field-level errors,
/// matching Symfony's validation error format.
///
/// ```json
/// {
///   "type": "https://httpstatuses.com/422",
///   "title": "Validation Failed",
///   "status": 422,
///   "detail": "The submitted data is invalid.",
///   "errors": {
///     "email": [{ "code": "email", "message": "Invalid email address" }],
///     "name": [{ "code": "length", "message": "Must be between 2 and 100 characters" }]
///   }
/// }
/// ```
#[derive(Debug)]
pub enum ValidationError {
    /// JSON deserialization failed
    InvalidJson(String),
    /// Query string parsing failed
    InvalidQuery(String),
    /// Validation constraints failed
    ValidationFailed(ValidationErrors),
}

#[derive(Serialize)]
struct ValidationErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    title: String,
    status: u16,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct FieldError {
    code: String,
    message: String,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        match self {
            ValidationError::InvalidJson(msg) => {
                let body = ValidationErrorResponse {
                    error_type: "https://httpstatuses.com/422".to_string(),
                    title: "Invalid Request Body".to_string(),
                    status: 422,
                    detail: msg,
                    errors: None,
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response()
            }
            ValidationError::InvalidQuery(msg) => {
                let body = ValidationErrorResponse {
                    error_type: "https://httpstatuses.com/422".to_string(),
                    title: "Invalid Query Parameters".to_string(),
                    status: 422,
                    detail: msg,
                    errors: None,
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response()
            }
            ValidationError::ValidationFailed(errors) => {
                // Convert validator errors to Symfony-style field errors
                let mut field_errors = serde_json::Map::new();

                for (field, errs) in errors.field_errors() {
                    let messages: Vec<FieldError> = errs
                        .iter()
                        .map(|e| FieldError {
                            code: e.code.to_string(),
                            message: e
                                .message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| format!("Validation failed: {}", e.code)),
                        })
                        .collect();
                    field_errors.insert(
                        field.to_string(),
                        serde_json::to_value(messages).unwrap_or_default(),
                    );
                }

                let body = ValidationErrorResponse {
                    error_type: "https://httpstatuses.com/422".to_string(),
                    title: "Validation Failed".to_string(),
                    status: 422,
                    detail: "The submitted data is invalid.".to_string(),
                    errors: Some(serde_json::Value::Object(field_errors)),
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response()
            }
        }
    }
}
