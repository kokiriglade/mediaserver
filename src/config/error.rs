use thiserror::Error;

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
