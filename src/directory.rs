use actix_files::Directory;
use actix_web::{HttpRequest, HttpResponse, dev::ServiceResponse, web::Data};
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use num_format::{Locale, ToFormattedString};
use percent_encoding::{CONTROLS, utf8_percent_encode};
use std::{cmp::Reverse, fs, io, path::Path, time::SystemTime};
use v_htmlescape::escape as escape_html_entity;

use crate::config::Config;

macro_rules! encode_file_url {
    ($path:expr) => {
        utf8_percent_encode($path, CONTROLS)
    };
}
macro_rules! encode_file_name {
    ($entry:expr) => {
        escape_html_entity(&$entry.file_name().to_string_lossy())
    };
}

fn sorted_entries(dir: &Directory) -> io::Result<Vec<fs::DirEntry>> {
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

pub fn directory_listing(dir: &Directory, req: &HttpRequest) -> Result<ServiceResponse, io::Error> {
    let config: &Data<Config> = req
        .app_data::<Data<Config>>()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Missing Config"))?;
    let dir_entries = sorted_entries(dir)?;

    let back_link: Option<String> = {
        let trimmed = req.path().trim_end_matches('/');
        let segments: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();

        if segments.len() <= 1 {
            None
        } else {
            let parent_path = segments[..segments.len() - 1].join("/");
            let href = format!("/{}/", parent_path);
            Some(escape_html_entity(&href).to_string())
        }
    };

    let items_str = dir_entries.len().to_formatted_string(&Locale::en);

    let title = format!("Index of {} ({} items)", req.path(), items_str);
    let mut list_items = String::new();
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
        let time_str = escape_html_entity(&raw_time);

        let is_dir = meta.is_dir();
        let emoji = config
            .file_listing_render
            .emoji
            .resolve_emoji(&entry, &meta);

        let name_html = if is_dir {
            format!(
                "<a class=\"filename\" href=\"{}/\">{}/</a>",
                encode_file_url!(&rel),
                encode_file_name!(entry)
            )
        } else {
            format!(
                "<a class=\"filename\" href=\"{}\">{}</a><span class=\"size\">{}</span>",
                encode_file_url!(&rel),
                encode_file_name!(entry),
                ByteSize::b(meta.len()).display().iec()
            )
        };

        list_items.push_str(&format!(
            "<li><span class=\"emoji\">{}</span><span class=\"timestamp\">{}</span>{}</li>\n",
            emoji, time_str, name_html
        ));
    }

    let back_html = if let Some(href) = back_link {
        format!(
            "<div class=\"back\"><a href=\"{}\">🔙 Parent directory</a></div>",
            href
        )
    } else {
        String::new()
    };

    let html = format!(
        "<!DOCTYPE html>
        <html lang=\"en\">
        <head>
            <meta charset=\"UTF-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
            <title>{title}</title>
            <style>
                :root {{
                    --bg: #121212;
                    --fg: #e0e0e0;
                    --accent: #81a1c1;
                    --muted: #888;
                    --border: #333;
                }}

                * {{ 
                    box-sizing: border-box; margin: 0; padding: 0; 
                }}
                
                body {{ 
                    background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; padding: 1rem; 
                }}
                
                .header {{
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    margin-bottom: 1rem;
                }}
                .header h1 {{ 
                    font-size: 1.5rem;
                    margin: 0; 
                }}
                
                ul {{ 
                    list-style: none;
                }}
                li {{ 
                    display: flex; 
                    align-items: center; 
                    padding: 0.5rem 0; 
                    border-bottom: 
                    1px solid var(--border); 
                }}
                
                .emoji {{ 
                    width: 2rem; 
                    text-align: center; 
                    user-select: none;
                }}
                .timestamp {{ 
                    width: 10rem; 
                    flex-shrink: 0; 
                    color: var(--muted); 
                    font-size: 0.9rem; 
                }}
                .filename {{ 
                    flex: 1; 
                    color: var(--accent); 
                    text-decoration: none; 
                }}
                .filename:hover {{ 
                    text-decoration: underline; 
                }}
                .size {{ 
                    margin-left: 1rem; 
                    flex-shrink: 0; 
                    color: var(--muted); 
                    font-size: 0.9rem; 
                }}
                
                .back {{
                    margin-bottom: 1rem;
                }}
                .back a {{
                    display: inline-block;
                    padding: 0.4rem 0.8rem;
                    background: var(--border);
                    color: var(--fg);
                    text-decoration: none;
                    border-radius: 0.25rem;
                    font-size: 0.9rem;
                }}
                .back a:hover {{
                    background: var(--accent);
                    color: var(--bg);
                }}
            </style>
        </head>
        <body>
            <div class=\"header\">
                <h1>{title}</h1>
                {back_html}
            </div>
            <ul>
                {list_items}
            </ul>
        </body>
        </html>"
    );

    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
    ))
}
