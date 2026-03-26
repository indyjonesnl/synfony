mod loader;
mod env;

pub use loader::ConfigLoader;

use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// The Synfony configuration system.
///
/// Mirrors Symfony's configuration approach:
/// - YAML config files in `config/`
/// - Environment variable interpolation (`${VAR}`, `${VAR:default}`)
/// - .env file cascading: .env → .env.local → .env.{APP_ENV} → .env.{APP_ENV}.local
///
/// # Example
/// ```ignore
/// let config = SynfonyConfig::load("config/")?;
/// let db: DatabaseConfig = config.section("database")?;
/// ```
pub struct SynfonyConfig {
    inner: config::Config,
}

impl SynfonyConfig {
    /// Load configuration from the given directory.
    ///
    /// This follows Symfony's loading order:
    /// 1. Load .env files (with cascading)
    /// 2. Load YAML config files from `config_dir`
    /// 3. Apply environment variable overrides
    pub fn load(project_root: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let root = project_root.as_ref();

        // Step 1: Load .env files with Symfony-style cascading
        let app_env = env::load_dotenv_cascade(root)?;

        // Step 2: Build config from YAML files
        let config_dir = root.join("config");
        let loader = ConfigLoader::new(&config_dir, &app_env)?;

        Ok(SynfonyConfig {
            inner: loader.build()?,
        })
    }

    /// Deserialize a configuration section into a typed struct.
    ///
    /// Equivalent to Symfony's config tree sections.
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct DatabaseConfig {
    ///     url: String,
    ///     pool_size: u32,
    /// }
    /// let db: DatabaseConfig = config.section("database")?;
    /// ```
    pub fn section<T: DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError> {
        self.inner
            .get::<T>(key)
            .map_err(|e| ConfigError::Section {
                key: key.to_string(),
                source: e,
            })
    }

    /// Get a single configuration value.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError> {
        self.inner
            .get::<T>(key)
            .map_err(|e| ConfigError::Section {
                key: key.to_string(),
                source: e,
            })
    }

    /// Get the current application environment (dev, test, prod).
    pub fn app_env(&self) -> String {
        std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string())
    }

    /// Check if debug mode is enabled.
    pub fn is_debug(&self) -> bool {
        std::env::var("APP_DEBUG")
            .map(|v| v == "true" || v == "1")
            .unwrap_or_else(|_| self.app_env() == "dev")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load .env file: {0}")]
    DotEnv(String),

    #[error("Failed to load config file '{path}': {source}")]
    FileLoad {
        path: PathBuf,
        source: config::ConfigError,
    },

    #[error("Failed to deserialize config section '{key}': {source}")]
    Section {
        key: String,
        source: config::ConfigError,
    },

    #[error("Configuration error: {0}")]
    Other(String),
}
