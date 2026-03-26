use axum::Router;
use synfony_config::SynfonyConfig;
use synfony_di::Container;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// The HTTP Kernel — equivalent to Symfony's HttpKernel.
///
/// Responsible for building the final Axum application with middleware.
pub struct Kernel {
    state: AppState,
    router: Router<AppState>,
}

impl Kernel {
    pub fn new(config: SynfonyConfig, container: Container) -> Self {
        let state = AppState::new(container, config);
        Kernel {
            state,
            router: Router::new(),
        }
    }

    /// Set the pre-built router (with all controller routes merged).
    pub fn with_router(mut self, router: Router<AppState>) -> Self {
        self.router = router;
        self
    }

    /// Add default middleware layers.
    pub fn with_default_middleware(self) -> Self {
        self
    }

    /// Build the final Axum application ready to serve.
    pub fn build(self) -> Router {
        let is_debug = self.state.config.is_debug();

        let mut app = self.router;

        if is_debug {
            app = app.layer(TraceLayer::new_for_http());
        }

        app = app.layer(CorsLayer::permissive());

        app.with_state(self.state)
    }
}
