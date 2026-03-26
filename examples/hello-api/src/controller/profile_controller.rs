use serde::Serialize;
use synfony::axum::extract::Json;
use synfony::prelude::*;
use synfony::security::CurrentUser;

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
}

pub struct ProfileController;

#[controller("/api")]
impl ProfileController {
    #[route(GET, "/me", name = "me")]
    async fn me(user: CurrentUser) -> Json<MeResponse> {
        Json(MeResponse {
            user_id: user.user_id().to_string(),
            email: user.user_identifier().to_string(),
            roles: user.roles().iter().cloned().collect(),
        })
    }
}
