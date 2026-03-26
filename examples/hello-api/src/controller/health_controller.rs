use synfony::axum::extract::Json;
use synfony::axum::routing::get;
use synfony::axum::Router;
use synfony::AppState;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    framework: String,
    version: String,
}

/// Health check controller — the simplest possible controller.
///
/// In the future, the #[controller] macro will generate this Router.
/// For now, we build it manually to demonstrate the pattern.
///
/// Symfony equivalent:
/// ```php
/// #[Route('/api')]
/// class HealthController extends AbstractController
/// {
///     #[Route('/health', methods: ['GET'])]
///     public function health(): JsonResponse { ... }
/// }
/// ```
pub struct HealthController;

impl HealthController {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/api/health", get(Self::health))
    }

    async fn health() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "ok".to_string(),
            framework: "Synfony".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
