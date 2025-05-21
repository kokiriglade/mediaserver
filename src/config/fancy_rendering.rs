use serde::{Deserialize, Serialize};

use super::FancyRendererEmojis;

#[derive(Deserialize, Default, Serialize, Debug, Clone)]
#[serde(default)]
pub struct FancyRendererConfig {
    pub emoji: FancyRendererEmojis,
}
