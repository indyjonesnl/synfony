use synfony_config::SynfonyConfig;
use synfony_di::Container;
use std::sync::Arc;

/// The application state shared across all Axum handlers.
///
/// This is the `S` in `Router<S>`. Every handler has access to
/// the DI container and configuration through this state.
///
/// Implements `AsRef<Container>` so that `Inject<T>` can resolve
/// services from any handler.
#[derive(Clone)]
pub struct AppState {
    pub container: Container,
    pub config: Arc<SynfonyConfig>,
}

impl AppState {
    pub fn new(container: Container, config: SynfonyConfig) -> Self {
        AppState {
            container,
            config: Arc::new(config),
        }
    }
}

impl AsRef<Container> for AppState {
    fn as_ref(&self) -> &Container {
        &self.container
    }
}
