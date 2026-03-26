use serde::{Deserialize, Serialize};
use synfony::axum::extract::Json;
use synfony::axum::http::StatusCode;
use synfony::axum::response::IntoResponse;
use synfony::di::Inject;
use synfony::prelude::*;
use synfony::security::SecurityToken;
use synfony_security::jwt::JwtManager;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    #[serde(rename = "type")]
    pub token_type: String,
}

pub struct LoginController;

#[controller("/api")]
impl LoginController {
    #[route(POST, "/login", name = "login")]
    async fn login(
        jwt_manager: Inject<JwtManager>,
        Json(payload): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, ApiError> {
        let (user_id, roles) = match (payload.email.as_str(), payload.password.as_str()) {
            ("admin@example.com", "admin") => ("1", vec!["ROLE_USER", "ROLE_ADMIN"]),
            ("user@example.com", "user") => ("2", vec!["ROLE_USER"]),
            _ => {
                return Err(ApiError::unauthorized("Invalid credentials"));
            }
        };

        let token = SecurityToken::new(user_id, &payload.email).with_roles(roles);

        let jwt = jwt_manager
            .generate(&token)
            .map_err(|e| ApiError::internal(e.to_string()))?;

        Ok((
            StatusCode::OK,
            Json(TokenResponse {
                token: jwt,
                token_type: "Bearer".to_string(),
            }),
        ))
    }
}
