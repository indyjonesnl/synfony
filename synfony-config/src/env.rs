use std::path::Path;

use crate::ConfigError;

/// Load .env files with Symfony-style cascading.
///
/// Loading order (each file overrides the previous):
/// 1. `.env`           — Default values, committed to VCS
/// 2. `.env.local`     — Local overrides, NOT committed
/// 3. `.env.{APP_ENV}` — Environment-specific defaults (e.g., .env.test)
/// 4. `.env.{APP_ENV}.local` — Local env-specific overrides
///
/// Returns the resolved APP_ENV value.
pub fn load_dotenv_cascade(root: &Path) -> Result<String, ConfigError> {
    // Load base .env first
    let base_env = root.join(".env");
    if base_env.exists() {
        dotenvy::from_path(&base_env).ok();
    }

    // Load .env.local (local overrides, gitignored)
    let local_env = root.join(".env.local");
    if local_env.exists() {
        dotenvy::from_path_override(&local_env).ok();
    }

    // Determine APP_ENV (default: "dev")
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    // Load .env.{APP_ENV}
    let env_specific = root.join(format!(".env.{}", app_env));
    if env_specific.exists() {
        dotenvy::from_path_override(&env_specific).ok();
    }

    // Load .env.{APP_ENV}.local
    let env_specific_local = root.join(format!(".env.{}.local", app_env));
    if env_specific_local.exists() {
        dotenvy::from_path_override(&env_specific_local).ok();
    }

    Ok(app_env)
}
