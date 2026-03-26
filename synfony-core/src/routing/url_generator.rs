use std::sync::Arc;

use super::registry::{RouteRegistry, RoutingError};

/// Generates URLs from named routes — Symfony's `UrlGeneratorInterface`.
///
/// Supports relative paths and absolute URLs. When `DEFAULT_URI` is set
/// (e.g., `https://example.com`), absolute URLs use that base — giving
/// production-like URLs in local development with a hosts file entry.
///
/// # Example
/// ```ignore
/// // In a handler:
/// async fn create(url_gen: Inject<UrlGenerator>) -> impl IntoResponse {
///     let path = url_gen.path("user_show", &[("id", "5")])?;
///     // → "/api/users/5"
///
///     let url = url_gen.url("user_show", &[("id", "5")])?;
///     // → "https://example.com/api/users/5" (with DEFAULT_URI)
///     // → "/api/users/5" (without DEFAULT_URI)
/// }
///
/// // In a CLI command or message handler (no request context needed):
/// let url_gen = container.resolve::<UrlGenerator>();
/// let link = url_gen.url("password_reset", &[("token", &token)])?;
/// ```
pub struct UrlGenerator {
    registry: Arc<RouteRegistry>,
    base_url: Option<String>,
}

impl UrlGenerator {
    pub fn new(registry: Arc<RouteRegistry>, base_url: Option<String>) -> Self {
        let base_url = base_url
            .filter(|u| !u.is_empty())
            .map(|u| u.trim_end_matches('/').to_string());
        UrlGenerator { registry, base_url }
    }

    /// Generate a relative path for the named route.
    ///
    /// Equivalent to `$router->generate('route', params, UrlGeneratorInterface::ABSOLUTE_PATH)`.
    pub fn path(&self, name: &str, params: &[(&str, &str)]) -> Result<String, RoutingError> {
        self.registry.generate_path(name, params)
    }

    /// Generate an absolute URL for the named route.
    ///
    /// Uses `DEFAULT_URI` as the base. Falls back to a relative path if not configured.
    ///
    /// Equivalent to `$router->generate('route', params, UrlGeneratorInterface::ABSOLUTE_URL)`.
    pub fn url(&self, name: &str, params: &[(&str, &str)]) -> Result<String, RoutingError> {
        let path = self.registry.generate_path(name, params)?;
        match &self.base_url {
            Some(base) => Ok(format!("{}{}", base, path)),
            None => Ok(path),
        }
    }

    /// Get the configured base URL (DEFAULT_URI), if any.
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    /// Get a reference to the underlying route registry.
    pub fn registry(&self) -> &RouteRegistry {
        &self.registry
    }
}
