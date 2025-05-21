use std::{cmp::Reverse, fs, io, path::Path, time::SystemTime};

use actix_files::Directory;
use actix_web::{HttpRequest, HttpResponse, dev::ServiceResponse, web::Data};
use askama::Template;
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use num_format::{Locale, ToFormattedString};
use template::{DirectoryView, IndividualListing};

use crate::config::Config;

pub mod template;

/// Reads the given directory and returns its entries, where directories are at
/// the beginning, and everything is sorted by their last modified timestamp
/// (where most recent is at the start)
pub fn sorted_entries(dir: &Directory) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(&dir.path)?
        .filter(|res| dir.is_visible(res))
        .filter_map(Result::ok)
        .collect();

    entries.sort_by_key(|entry| {
        // try to get metadata
        // if that fails, treat as file and epoch time.
        let meta = entry.metadata().ok();

        // directories first - `is_file = false` for dirs
        let is_file = meta.as_ref().map(|m| !m.is_dir()).unwrap_or(true);

        // pull modified time out of the result, defaulting to the unix epoch
        let modified = meta
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        (is_file, Reverse(modified))
    });

    Ok(entries)
}

/// Renders a given directory to a nice HTML structure using askama.
/// Referred to as the "fancy renderer" in configuration.
pub fn directory_listing(dir: &Directory, req: &HttpRequest) -> Result<ServiceResponse, io::Error> {
    let config: &Data<Config> = req
        .app_data::<Data<Config>>()
        .ok_or_else(|| io::Error::other("Missing Config"))?;
    let dir_entries = sorted_entries(dir)?;

    let back_link: Option<String> = {
        let trimmed = req.path().trim_end_matches('/');
        let segments: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();

        if segments.len() <= 1 {
            None
        } else {
            let parent_path = segments[..segments.len() - 1].join("/");
            let href = format!("/{}/", parent_path);
            Some(href)
        }
    };

    let items_str = dir_entries.len().to_formatted_string(&Locale::en);

    let mut list_items = Vec::<IndividualListing>::with_capacity(dir_entries.len());
    let base = Path::new(req.path());

    for entry in dir_entries {
        let rel = match entry.path().strip_prefix(&dir.path) {
            Ok(p) if cfg!(windows) => base.join(p).to_string_lossy().replace('\\', "/"),
            Ok(p) => base.join(p).to_string_lossy().into_owned(),
            Err(_) => continue,
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let modified = meta.modified().unwrap_or(SystemTime::now());
        let datetime: DateTime<Utc> = modified.into();
        let raw_time = datetime.format("%Y-%m-%d %H:%M").to_string();

        let is_dir = meta.is_dir();
        let emoji = config
            .file_listing_render
            .emoji
            .resolve_emoji(&entry, &meta);

        let byte_size = ByteSize::b(meta.len()).display().iec().to_string();

        list_items.push(IndividualListing {
            emoji: emoji,
            timestamp: raw_time,
            file_href: rel,
            file_name: entry.file_name().to_string_lossy().to_string(),
            byte_size: byte_size,
            is_directory: is_dir,
        });
    }

    let directory_view = DirectoryView {
        current_directory: req.path(),
        total_items: &items_str,
        parent_dir_href: &back_link.unwrap_or(String::new()),
        individual_listings: &list_items,
    };

    let html = directory_view.render();

    if html.is_err() {
        Ok(ServiceResponse::new(
            req.clone(),
            HttpResponse::InternalServerError().body("Failed to render directory listing"),
        ))
    } else {
        Ok(ServiceResponse::new(
            req.clone(),
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html.unwrap()),
        ))
    }
}
