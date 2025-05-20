use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
    path::PathBuf,
    str::FromStr,
};

use log::info;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub web_server: WebServerConfig,
    pub storage: StorageConfig,
    pub namespaces: HashMap<String, NamespaceDefinition>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web_server: WebServerConfig::default(),
            storage: StorageConfig::default(),
            namespaces: HashMap::from([("ferris".to_string(), NamespaceDefinition::default())]),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error while checking if `{path}` exists: {source}")]
    IoExists {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("I/O error while reading `{path}`: {source}")]
    IoRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("I/O error while writing `{path}`: {source}")]
    IoWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse `{path}` as TOML: {source}")]
    TomlParse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to serialize as TOML: {source}")]
    TomlWrite {
        #[source]
        source: toml::ser::Error,
    },
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

            fs::write(CONFIG_PATH, serialized).map_err(|e| ConfigError::IoWrite {
                path: CONFIG_PATH.into(),
                source: e,
            })?;

            info!("Done! Edit the config file and start again");

            std::process::exit(0);
        }

        let content = fs::read_to_string(CONFIG_PATH).map_err(|e| ConfigError::IoRead {
            path: CONFIG_PATH.into(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| ConfigError::TomlParse {
            path: CONFIG_PATH.into(),
            source: e,
        })
    }

    pub fn get_uploads_path(&self) -> PathBuf {
        PathBuf::from(&self.storage.uploads_directory)
    }

    pub fn get_temp_path(&self) -> PathBuf {
        self.get_uploads_path().join(".temp")
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(with = "url_serde")]
    pub listen_url: Url,
    #[serde(with = "url_serde")]
    pub redirect_index_to: Url,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
            listen_url: Url::from_str("http://localhost:3000")
                .expect("default 'url' should be parseable"),
            redirect_index_to: Url::from_str("http://github.com/kokiriglade/mediaserver")
                .expect("default 'redirect_index_to' should be parseable"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct StorageConfig {
    pub default_namespace_fs_path: String,
    pub target_file_name_length: u32,
    pub max_file_size_bytes: usize,
    pub uploads_directory: String,
    pub max_attempts_before_grow: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            default_namespace_fs_path: "ferris".to_string(),
            target_file_name_length: 1,
            max_attempts_before_grow: 32,
            max_file_size_bytes: 1024 * 1024 * 100,
            uploads_directory: "uploads".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct NamespaceDefinition {
    pub file_system_path: String,
    pub key: String,
    pub file_listing: FileListingConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct FileListingConfig {
    pub show: bool,
    pub use_fancy_renderer: bool,
}

impl Default for FileListingConfig {
    fn default() -> Self {
        Self {
            show: false,
            use_fancy_renderer: true,
        }
    }
}

fn create_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

impl NamespaceDefinition {
    pub fn auth<'a>(
        namespaces: &'a HashMap<String, NamespaceDefinition>,
        namespace: &'a String,
        key: &'a String,
    ) -> Option<&'a NamespaceDefinition> {
        let namespace = match namespaces.get(namespace) {
            Some(ns) => ns,
            None => {
                return None;
            }
        };

        if &namespace.key != key {
            return None;
        }

        Some(namespace)
    }

    pub fn get_path(&self, config: &Config) -> PathBuf {
        config.get_uploads_path().join(&self.file_system_path)
    }

    pub fn create_random_file_name(
        &self,
        config: &Config,
        file_extension: &str,
    ) -> Result<PathBuf, io::Error> {
        let base_dir = self.get_path(config);
        fs::create_dir_all(&base_dir)?;

        let mut length = config.storage.target_file_name_length as usize;
        let mut attempts = 0;

        loop {
            // create a candidate
            let candidate_name: String = create_random_string(length);
            let candidate = base_dir.join(format!("{candidate_name}.{file_extension}"));

            // attempt to reserve it atomically
            let open_result = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate);

            match open_result {
                // we've both checked for existence and created it
                Ok(_file) => return Ok(candidate),

                // it already exists - retry
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    attempts += 1;
                    // after N failed attempts we should bump the length
                    if attempts > config.storage.max_attempts_before_grow {
                        length += 1;
                        attempts = 0;
                    }
                    continue;
                }

                // any other error then bail immediately
                Err(e) => return Err(e),
            }
        }
    }
}

impl Default for NamespaceDefinition {
    fn default() -> Self {
        Self {
            file_system_path: "ferris".to_string(),
            key: create_random_string(128),
            file_listing: FileListingConfig::default(),
        }
    }
}
