use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use clap::{Parser, Subcommand};
use synfony_config::SynfonyConfig;
use synfony_console::ConsoleIO;
use synfony_di::Container;
use synfony_security::firewall::FirewallLayer;
use tokio::net::TcpListener;

use crate::kernel::Kernel;
use crate::routing::{ControllerRegistration, RouteRegistry, UrlGenerator};
use crate::state::AppState;

/// The Synfony Application — the main entry point.
///
/// Controllers are auto-discovered via the `#[controller]` macro — no manual
/// route registration needed. Just create a controller file, add routes,
/// and the framework finds it automatically.
///
/// # Example
/// ```ignore
/// #[tokio::main]
/// async fn main() {
///     let mut app = Application::new().unwrap();
///     app.register_service(Arc::new(my_service));
///     app.set_firewall(firewall_layer);
///     app.run().await.unwrap();
///     // Controllers are auto-discovered — no register_routes() calls needed
/// }
/// ```
pub struct Application {
    config: SynfonyConfig,
    container: Container,
    firewall: Option<FirewallLayer>,
}

#[derive(Parser)]
#[command(name = "synfony", about = "Synfony Application")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Address to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "8000")]
        port: u16,
    },

    /// Show all registered routes
    #[command(name = "debug:router")]
    DebugRouter,

    /// Show all registered services in the container
    #[command(name = "debug:container")]
    DebugContainer,
}

impl Application {
    /// Create a new application, loading configuration from the current directory.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();

        let config = SynfonyConfig::load(".")?;
        let container = Container::new();

        Ok(Application {
            config,
            container,
            firewall: None,
        })
    }

    /// Create with a specific project root directory.
    pub fn with_root(root: impl Into<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let root = root.into();

        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();

        let config = SynfonyConfig::load(&root)?;
        let container = Container::new();

        Ok(Application {
            config,
            container,
            firewall: None,
        })
    }

    /// Get a reference to the DI container.
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &SynfonyConfig {
        &self.config
    }

    /// Register a service in the DI container.
    pub fn register_service<T: 'static + Send + Sync>(&self, service: Arc<T>) {
        self.container.set(service);
    }

    /// Set the security firewall layer (applied globally to all routes).
    ///
    /// The firewall's pattern matching handles which routes are public vs protected.
    /// Equivalent to Symfony's security.yaml firewall configuration.
    pub fn set_firewall(&mut self, firewall: FirewallLayer) {
        self.firewall = Some(firewall);
    }

    /// Run the application — either serve HTTP or execute a CLI command.
    ///
    /// Controllers are auto-discovered via `inventory`. Any struct with
    /// `#[controller]` is automatically registered.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Auto-discover all controllers registered via #[controller] macro
        let mut registry = RouteRegistry::new();
        let mut router = Router::new();

        let mut controller_count = 0;
        for registration in inventory::iter::<ControllerRegistration> {
            let routes = (registration.routes_fn)();
            let metadata = (registration.metadata_fn)();

            router = router.merge(routes);
            for def in metadata {
                registry.add(def);
            }
            controller_count += 1;
        }

        tracing::debug!(
            controllers = controller_count,
            routes = registry.len(),
            "Auto-discovered controllers"
        );

        // Apply firewall globally if configured
        if let Some(firewall) = &self.firewall {
            router = router.layer(firewall.clone());
        }

        // Freeze registry and create UrlGenerator
        let registry = Arc::new(registry);
        let base_url = std::env::var("DEFAULT_URI").ok();
        let url_generator = Arc::new(UrlGenerator::new(registry.clone(), base_url));
        self.container.set(registry.clone());
        self.container.set(url_generator);

        let cli = Cli::parse();

        match cli.command {
            Some(Commands::Serve { host, port }) => {
                self.serve(router, &host, port).await
            }
            Some(Commands::DebugRouter) => {
                Self::debug_router(&registry, &self.container);
                Ok(())
            }
            Some(Commands::DebugContainer) => {
                Self::debug_container();
                Ok(())
            }
            None => {
                self.serve(router, "0.0.0.0", 8000).await
            }
        }
    }

    async fn serve(
        self,
        router: Router<AppState>,
        host: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let io = ConsoleIO::new();

        let kernel = Kernel::new(self.config, self.container)
            .with_router(router)
            .with_default_middleware();

        let app = kernel.build();

        let addr: SocketAddr = format!("{host}:{port}").parse()?;
        let listener = TcpListener::bind(addr).await?;

        io.title("Synfony Development Server");
        io.success(&format!("Listening on http://{addr}"));
        io.comment("Press Ctrl+C to stop the server");
        io.newline();

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn debug_router(registry: &RouteRegistry, container: &Container) {
        let io = ConsoleIO::new();
        io.title("Registered Routes");

        let routes = registry.all();

        if routes.is_empty() {
            io.comment("No named routes registered.");
            io.info("Use #[controller] and #[route] macros to define routes.");
            return;
        }

        let rows: Vec<Vec<&str>> = routes
            .iter()
            .map(|r| vec![r.name.as_str(), r.method.as_str(), r.path.as_str()])
            .collect();

        io.table(vec!["Name", "Method", "Path"], rows);

        let url_gen = container.resolve::<UrlGenerator>();
        if let Some(base) = url_gen.base_url() {
            io.newline();
            io.info(&format!("DEFAULT_URI: {}", base));
        }
    }

    fn debug_container() {
        let io = ConsoleIO::new();
        io.title("Registered Services");
        io.comment("Container debugging will be enhanced with service metadata collection.");
    }
}
