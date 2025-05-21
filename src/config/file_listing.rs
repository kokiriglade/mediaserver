use serde::{Deserialize, Serialize};

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
