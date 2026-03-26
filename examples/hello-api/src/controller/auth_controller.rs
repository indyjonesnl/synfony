use serde::{Deserialize, Serialize};
use synfony::axum::extract::Json;
use synfony::axum::http::StatusCode;
use synfony::axum::response::IntoResponse;
use synfony::axum::routing::{get, post};
use synfony::axum::Router;
use synfony::di::Inject;
use synfony::security::{CurrentUser, SecurityToken};
use synfony::AppState;
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

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
}

pub struct AuthController;

impl AuthController {
    /// Public routes (no auth required)
    pub fn public_routes() -> Router<AppState> {
        Router::new().route("/api/login", post(Self::login))
    }

    /// Protected routes (require valid JWT)
    pub fn protected_routes() -> Router<AppState> {
        Router::new().route("/api/me", get(Self::me))
    }

    /// POST /api/login — Authenticate and receive a JWT token.
    async fn login(
        jwt_manager: Inject<JwtManager>,
        Json(payload): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, synfony::ApiError> {
        // Simple demo authentication — in production, verify against database
        let (user_id, roles) = match (payload.email.as_str(), payload.password.as_str()) {
            ("admin@example.com", "admin") => ("1", vec!["ROLE_USER", "ROLE_ADMIN"]),
            ("user@example.com", "user") => ("2", vec!["ROLE_USER"]),
            _ => {
                return Err(synfony::ApiError::unauthorized("Invalid credentials"));
            }
        };

        let token = SecurityToken::new(user_id, &payload.email).with_roles(roles);

        let jwt = jwt_manager
            .generate(&token)
            .map_err(|e| synfony::ApiError::internal(e.to_string()))?;

        Ok((
            StatusCode::OK,
            Json(TokenResponse {
                token: jwt,
                token_type: "Bearer".to_string(),
            }),
        ))
    }

    /// GET /api/me — Return the current authenticated user's info.
    async fn me(user: CurrentUser) -> Json<MeResponse> {
        Json(MeResponse {
            user_id: user.user_id().to_string(),
            email: user.user_identifier().to_string(),
            roles: user.roles().iter().cloned().collect(),
        })
    }
}
