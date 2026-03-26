mod registry;
mod url_generator;

pub use registry::{RouteDefinition, RouteRegistry, RoutingError};
pub use url_generator::UrlGenerator;

use axum::Router;
use crate::state::AppState;

/// The Controller trait — implemented by the `#[controller]` macro.
///
/// This is the single source of truth for route definitions.
/// The macro generates both the Axum router and route metadata from
/// `#[route]` attributes, so routes are defined exactly once.
///
/// Equivalent to a Symfony controller class with `#[Route]` attributes.
pub trait Controller {
    /// Returns the Axum router with all routes for this controller.
    fn routes() -> Router<AppState>;

    /// Returns route metadata (name, path, method) for all named routes.
    fn route_metadata() -> Vec<RouteDefinition>;
}

/// Auto-registration entry for controller discovery via `inventory`.
///
/// The `#[controller]` macro submits one of these for each controller.
/// `Application::run()` collects them all automatically.
pub struct ControllerRegistration {
    pub name: &'static str,
    pub routes_fn: fn() -> Router<AppState>,
    pub metadata_fn: fn() -> Vec<RouteDefinition>,
}

inventory::collect!(ControllerRegistration);
