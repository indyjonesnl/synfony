use std::net::SocketAddr;

use axum::Router;
use clap::{Parser, Subcommand};
use synfony_config::SynfonyConfig;
use synfony_console::ConsoleIO;
use synfony_di::Container;
use tokio::net::TcpListener;

use crate::kernel::Kernel;
use crate::state::AppState;

/// The Synfony Application — the main entry point.
///
/// Equivalent to Symfony's Kernel + bin/console combined.
/// Handles both HTTP serving and CLI command execution.
///
/// # Example
/// ```ignore
/// #[tokio::main]
/// async fn main() {
///     let app = Application::new().await.unwrap();
///     app.register_routes(UserController::routes());
///     app.run().await.unwrap();
/// }
/// ```
pub struct Application {
    config: SynfonyConfig,
    container: Container,
    routes: Vec<Router<AppState>>,
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
        // Initialize tracing
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
            routes: Vec::new(),
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
            routes: Vec::new(),
        })
    }

    /// Get a mutable reference to the DI container for registering services.
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &SynfonyConfig {
        &self.config
    }

    /// Register a service in the DI container.
    pub fn register_service<T: 'static + Send + Sync>(&self, service: std::sync::Arc<T>) {
        self.container.set(service);
    }

    /// Register controller routes.
    pub fn register_routes(&mut self, routes: Router<AppState>) {
        self.routes.push(routes);
    }

    /// Run the application — either serve HTTP or execute a CLI command.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();

        match cli.command {
            Some(Commands::Serve { host, port }) => {
                self.serve(&host, port).await
            }
            Some(Commands::DebugRouter) => {
                self.debug_router();
                Ok(())
            }
            Some(Commands::DebugContainer) => {
                self.debug_container();
                Ok(())
            }
            None => {
                // Default: serve on port 8000
                self.serve("0.0.0.0", 8000).await
            }
        }
    }

    async fn serve(self, host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let io = ConsoleIO::new();

        let mut kernel = Kernel::new(self.config, self.container);
        for routes in self.routes {
            kernel = kernel.register_routes(routes);
        }

        let app = kernel.with_default_middleware().build();

        let addr: SocketAddr = format!("{host}:{port}").parse()?;
        let listener = TcpListener::bind(addr).await?;

        io.title("Synfony Development Server");
        io.success(&format!("Listening on http://{addr}"));
        io.comment("Press Ctrl+C to stop the server");
        io.newline();

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn debug_router(&self) {
        let io = ConsoleIO::new();
        io.title("Registered Routes");
        io.comment("Route debugging will be enhanced with route metadata collection.");
        io.info("Use #[controller] and #[route] macros to register routes.");
    }

    fn debug_container(&self) {
        let io = ConsoleIO::new();
        io.title("Registered Services");
        io.comment("Container debugging will be enhanced with service metadata collection.");
    }
}
