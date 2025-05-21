use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct StorageConfig {
    pub default_namespace_fs_path: String,
    pub max_file_size_bytes: usize,
    pub uploads_directory: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            default_namespace_fs_path: "ferris".to_string(),
            max_file_size_bytes: 1024 * 1024 * 100,
            uploads_directory: "uploads".to_string(),
        }
    }
}
