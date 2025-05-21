use std::{
    collections::HashMap,
    fs::{DirEntry, Metadata},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct FancyRendererEmojis {
    pub directory: String,
    pub unknown: String,
    pub file_extensions: HashMap<String, String>,
}

impl FancyRendererEmojis {
    pub fn resolve_emoji(
        &self,
        dir_entry: &DirEntry,
        metadata: &Metadata,
    ) -> String {
        let is_dir = metadata.is_dir();
        let ext_string = dir_entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        if is_dir {
            self.directory.clone()
        } else {
            self.file_extensions
                .get(ext_string.as_str())
                .cloned()
                .unwrap_or(self.unknown.clone())
        }
    }
}

impl Default for FancyRendererEmojis {
    fn default() -> Self {
        let file_extensions = HashMap::from([
            (String::from("png"), String::from("🖼️")),
            (String::from("jpg"), String::from("🖼️")),
            (String::from("jpeg"), String::from("🖼️")),
            (String::from("webp"), String::from("🖼️")),
            (String::from("gif"), String::from("🖼️")),
            (String::from("avif"), String::from("🖼️")),
            // ciff is an image file format I made a few years ago that nobody,
            // not even I, use - I just have it on my private instance.
            (String::from("ciff"), String::from("🖼️")),
            (String::from("mp4"), String::from("📺")),
            (String::from("zip"), String::from("📦")),
            (String::from("tar"), String::from("📦")),
            (String::from("rar"), String::from("📦")),
            (String::from("7z"), String::from("📦")),
            (String::from("gz"), String::from("📦")),
            (String::from("mp3"), String::from("🎵")),
            (String::from("wav"), String::from("🎵")),
            (String::from("ogg"), String::from("🎵")),
            // Modrinth Minecraft modpack file.
            (String::from("mrpack"), String::from("🕹️")),
            (String::from("md"), String::from("📄")),
            (String::from("txt"), String::from("📄")),
            (String::from("pdf"), String::from("📄")),
            (String::from("docx"), String::from("📄")),
            (String::from("log"), String::from("📄")),
            (String::from("json"), String::from("📄")),
            (String::from("jsonc"), String::from("📄")),
            (String::from("jar"), String::from("🍵")),
            (String::from("js"), String::from("🧩")),
            (String::from("rs"), String::from("🧩")),
            (String::from("css"), String::from("👕")),
            (String::from("exe"), String::from("💾")),
        ]);

        Self {
            directory: "📂".to_string(),
            unknown: "❓".to_string(),
            file_extensions,
        }
    }
}
