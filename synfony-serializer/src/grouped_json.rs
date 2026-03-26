use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// JSON response that applies serialization groups.
///
/// Wraps a pre-serialized `serde_json::Value` (produced by `serialize_group()`)
/// and returns it as a JSON response.
///
/// # Example
/// ```ignore
/// #[route(GET, "/users")]
/// async fn list(repo: Inject<UserRepo>) -> GroupedJson {
///     let users = repo.find_all().await.unwrap();
///     let values: Vec<_> = users.iter()
///         .map(|u| u.serialize_group("list").unwrap())
///         .collect();
///     GroupedJson::array(values)
/// }
///
/// #[route(GET, "/users/:id")]
/// async fn show(Path(id): Path<i32>, repo: Inject<UserRepo>) -> GroupedJson {
///     let user = repo.find(id).await.unwrap();
///     GroupedJson::value(user.serialize_group("detail").unwrap())
/// }
/// ```
pub struct GroupedJson {
    value: serde_json::Value,
    status: StatusCode,
}

impl GroupedJson {
    /// Create from a single serialized value.
    pub fn value(value: serde_json::Value) -> Self {
        GroupedJson {
            value,
            status: StatusCode::OK,
        }
    }

    /// Create from a vec of serialized values (for list endpoints).
    pub fn array(values: Vec<serde_json::Value>) -> Self {
        GroupedJson {
            value: serde_json::Value::Array(values),
            status: StatusCode::OK,
        }
    }

    /// Set a custom HTTP status code.
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl IntoResponse for GroupedJson {
    fn into_response(self) -> Response {
        (self.status, axum::Json(self.value)).into_response()
    }
}
