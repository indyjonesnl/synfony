use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::Deserialize;
use std::time::Duration;

use crate::OrmError;

/// Database configuration (loaded from config/app.yaml `database` section).
///
/// Equivalent to Doctrine's `DATABASE_URL` and connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Connection URL (e.g., "sqlite:./database.db", "postgres://user:pass@host/db")
    pub url: String,

    /// Maximum connections in the pool (default: 5)
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds (default: 5)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Enable SQL query logging (default: false)
    #[serde(default)]
    pub logging: bool,
}

fn default_pool_size() -> u32 {
    5
}

fn default_timeout() -> u64 {
    5
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            url: "sqlite:./database.db?mode=rwc".to_string(),
            pool_size: 5,
            timeout: 5,
            logging: false,
        }
    }
}

/// Establish a database connection from config.
///
/// Equivalent to Doctrine's connection factory.
///
/// ```ignore
/// let config = DatabaseConfig { url: "sqlite:./db.sqlite?mode=rwc".into(), ..Default::default() };
/// let db = synfony_orm::connect(&config).await?;
/// ```
pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection, OrmError> {
    let mut opts = ConnectOptions::new(&config.url);
    opts.max_connections(config.pool_size)
        .connect_timeout(Duration::from_secs(config.timeout))
        .sqlx_logging(config.logging);

    Database::connect(opts)
        .await
        .map_err(|e| OrmError::Connection(e.to_string()))
}
