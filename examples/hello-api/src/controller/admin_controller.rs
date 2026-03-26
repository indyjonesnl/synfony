use serde::Serialize;
use synfony::axum::extract::Json;
use synfony::axum::routing::get;
use synfony::axum::Router;
use synfony::security::CurrentUser;
use synfony::{ApiError, AppState};

#[derive(Serialize)]
pub struct AdminDashboard {
    pub message: String,
    pub user: String,
    pub total_users: u32,
}

/// Admin controller — protected by ROLE_ADMIN via access_control rules.
///
/// Symfony equivalent:
/// ```php
/// #[Route('/api/admin')]
/// #[IsGranted('ROLE_ADMIN')]
/// class AdminController extends AbstractController
/// {
///     #[Route('/dashboard', methods: ['GET'])]
///     public function dashboard(): JsonResponse { ... }
/// }
/// ```
pub struct AdminController;

impl AdminController {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/api/admin/dashboard", get(Self::dashboard))
    }

    /// GET /api/admin/dashboard — Admin-only endpoint.
    ///
    /// Access is controlled by the firewall's access_control rules:
    /// `/api/admin/*` requires ROLE_ADMIN.
    async fn dashboard(user: CurrentUser) -> Result<Json<AdminDashboard>, ApiError> {
        // The firewall already checked ROLE_ADMIN via access_control,
        // but we can also check programmatically:
        if !user.has_role("ROLE_ADMIN") {
            return Err(ApiError::forbidden("Admin access required"));
        }

        Ok(Json(AdminDashboard {
            message: "Welcome to the admin dashboard".to_string(),
            user: user.user_identifier().to_string(),
            total_users: 42,
        }))
    }
}
