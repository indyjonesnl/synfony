use serde::Serialize;
use synfony::axum::extract::Json;
use synfony::prelude::*;
use synfony::security::CurrentUser;

#[derive(Serialize)]
pub struct AdminDashboard {
    pub message: String,
    pub user: String,
    pub total_users: u32,
}

pub struct AdminController;

#[controller("/api/admin")]
impl AdminController {
    #[route(GET, "/dashboard", name = "admin_dashboard")]
    async fn dashboard(user: CurrentUser) -> Result<Json<AdminDashboard>, ApiError> {
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
