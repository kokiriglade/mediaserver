mod error;
mod fancy_rendering;
mod fancy_rendering_emoji;
mod file_listing;
mod namespace_def;
mod random_string_generator;
mod storage;
mod web_server;

use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::PathBuf};

pub use error::ConfigError;
pub use fancy_rendering::FancyRendererConfig;
pub use fancy_rendering_emoji::FancyRendererEmojis;
pub use file_listing::FileListingConfig;
pub use namespace_def::NamespaceDefinition;
pub use random_string_generator::StringGenerator;
pub use storage::StorageConfig;
pub use web_server::WebServerConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub web_server: WebServerConfig,
    pub file_listing_render: FancyRendererConfig,
    pub storage: StorageConfig,
    pub namespaces: HashMap<String, NamespaceDefinition>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web_server: WebServerConfig::default(),
            file_listing_render: FancyRendererConfig::default(),
            storage: StorageConfig::default(),
            namespaces: HashMap::from([(
                "ferris".to_string(),
                NamespaceDefinition::default(),
            )]),
        }
    }
}

static CONFIG_PATH: &str = "mediaserver.toml";

impl Config {
    pub fn read() -> Result<Config, ConfigError> {
        if !fs::exists(CONFIG_PATH).map_err(|e| ConfigError::IoExists {
            path: CONFIG_PATH.into(),
            source: e,
        })? {
            info!(
                "Failed to find `{}`; writing default config...",
                CONFIG_PATH
            );

            let serialized = toml::to_string_pretty(&Self::default())
                .map_err(|e| ConfigError::TomlWrite { source: e })?;

            fs::write(CONFIG_PATH, serialized).map_err(|e| {
                ConfigError::IoWrite {
                    path: CONFIG_PATH.into(),
                    source: e,
                }
            })?;

            info!("Done! Edit the config file and start again");

            std::process::exit(0);
        }

        let content = fs::read_to_string(CONFIG_PATH).map_err(|e| {
            ConfigError::IoRead {
                path: CONFIG_PATH.into(),
                source: e,
            }
        })?;

        toml::from_str(&content).map_err(|e| ConfigError::TomlParse {
            path: CONFIG_PATH.into(),
            source: e,
        })
    }

    pub fn create_uploads_directory(&self) -> io::Result<()> {
        let uploads_path: PathBuf = self.get_uploads_path();
        if !fs::exists(&uploads_path)? {
            fs::create_dir_all(&uploads_path)?;
        }

        let temp_path = self.get_temp_path();
        if !fs::exists(&temp_path)? {
            fs::create_dir(temp_path)?;
        }

        for namespace in &self.namespaces {
            let namespace_path = namespace.1.get_path(self);

            if !fs::exists(&namespace_path)? {
                fs::create_dir(&namespace_path)?;
                info!(
                    "Created namespace directory: '{}'",
                    &namespace_path.display()
                );
            }
        }

        Ok(())
    }

    pub fn get_uploads_path(&self) -> PathBuf {
        PathBuf::from(&self.storage.uploads_directory)
    }

    pub fn get_temp_path(&self) -> PathBuf {
        self.get_uploads_path().join(".temp")
    }
}
