use synfony::axum::extract::Json;
use synfony::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    framework: String,
    version: String,
}

pub struct HealthController;

#[controller("/api")]
impl HealthController {
    #[route(GET, "/health", name = "health")]
    async fn health() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "ok".to_string(),
            framework: "Synfony".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
