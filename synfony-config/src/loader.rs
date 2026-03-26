use config::{Config, File, FileFormat, Environment};
use std::path::{Path, PathBuf};

use crate::ConfigError;

/// Loads and merges configuration files from the config/ directory.
///
/// Follows Symfony conventions:
/// - `config/app.yaml` — main application config
/// - `config/packages/*.yaml` — package-specific config (future)
/// - Environment variables override YAML values
pub struct ConfigLoader {
    builder: config::ConfigBuilder<config::builder::DefaultState>,
}

impl ConfigLoader {
    pub fn new(config_dir: &Path, app_env: &str) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // Load main app config
        let app_config = config_dir.join("app.yaml");
        if app_config.exists() {
            builder = builder.add_source(
                File::from(app_config.clone())
                    .format(FileFormat::Yaml)
                    .required(false),
            );
        }

        // Load environment-specific config (e.g., config/app.dev.yaml)
        let env_config = config_dir.join(format!("app.{}.yaml", app_env));
        if env_config.exists() {
            builder = builder.add_source(
                File::from(env_config)
                    .format(FileFormat::Yaml)
                    .required(false),
            );
        }

        // Load all files from config/packages/ if it exists
        let packages_dir = config_dir.join("packages");
        if packages_dir.exists() && packages_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&packages_dir) {
                let mut files: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension()
                            .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    })
                    .collect();
                files.sort();

                for file in files {
                    builder = builder.add_source(
                        File::from(file)
                            .format(FileFormat::Yaml)
                            .required(false),
                    );
                }
            }
        }

        // Environment variables override everything.
        // APP__DATABASE__URL maps to database.url (double underscore = separator)
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

        Ok(ConfigLoader { builder })
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        self.builder.build().map_err(|e| ConfigError::Other(e.to_string()))
    }
}
