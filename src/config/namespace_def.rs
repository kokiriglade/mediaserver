use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::{Config, FileListingConfig, StringGenerator};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct NamespaceDefinition {
    pub file_system_path: String,
    pub key: String,
    pub file_listing: FileListingConfig,
    pub file_name_generator: StringGenerator,
}

impl NamespaceDefinition {
    pub fn auth<'a>(
        namespaces: &'a HashMap<String, NamespaceDefinition>,
        namespace: &'a String,
        key: &'a String,
    ) -> Option<&'a NamespaceDefinition> {
        let namespace = namespaces.get(namespace)?;

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

        match &self.file_name_generator {
            // For Random, pull out the starting length...
            StringGenerator::Random {
                length: initial_len,
                max_attempts_before_grow,
            } => {
                let mut length = *initial_len;
                let mut attempts = 0;

                loop {
                    // build a fresh generator each time with the current length
                    let string_gen = StringGenerator::Random {
                        length,
                        max_attempts_before_grow: *max_attempts_before_grow,
                    };
                    let candidate_name = string_gen.generate();
                    let candidate = base_dir
                        .join(format!("{candidate_name}.{file_extension}"));

                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&candidate)
                    {
                        Ok(_) => return Ok(candidate),
                        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                            attempts += 1;
                            if &attempts > string_gen.max_attempts_before_grow()
                            {
                                length += 1; // bump length
                                attempts = 0; // reset counter
                            }
                            // retry with the (possibly) new length
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            // chances of a collision are near-zero
            StringGenerator::Uuid => {
                let file_name = self.file_name_generator.generate();
                Ok(base_dir.join(format!("{file_name}.{file_extension}")))
            }
        }
    }
}

impl Default for NamespaceDefinition {
    fn default() -> Self {
        Self {
            file_system_path: "ferris".to_string(),
            key: StringGenerator::Random {
                length: 128,
                max_attempts_before_grow: 0,
            }
            .generate(),
            file_listing: FileListingConfig::default(),
            file_name_generator: StringGenerator::Random {
                length: 12,
                max_attempts_before_grow: 0,
            },
        }
    }
}
