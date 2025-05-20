mod config;
mod directory;
mod routes;

use actix_files::Files;
use actix_multipart::form::{MultipartFormConfig, tempfile::TempFileConfig};
use actix_web::{
    App, HttpResponse, HttpServer,
    http::header,
    web::{self, Data},
};
use config::Config;
use directory::directory_listing;
use log::{LevelFilter, error, info};
use routes::upload;
use std::{fs, io};

async fn index_redirect(cfg: Data<Config>) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .insert_header((
            header::LOCATION,
            cfg.web_server.redirect_index_to.to_string(),
        ))
        .finish()
}

fn create_uploads_directory(config: &Data<Config>) -> io::Result<()> {
    let uploads_path = config.get_uploads_path();
    if !fs::exists(&uploads_path)? {
        fs::create_dir_all(&uploads_path)?;
    }

    let temp_path = config.get_temp_path();
    if !fs::exists(&temp_path)? {
        fs::create_dir(temp_path)?;
    }

    for namespace in &config.namespaces {
        let namespace_path = namespace.1.get_path(config);

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

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("actix_server", LevelFilter::Off)
        .init();

    let config = Data::new(Config::read().unwrap_or_else(|e| {
        error!("Failed to read configuration: {}", e);
        std::process::exit(1);
    }));

    info!(
        "Found {} upload namespace{}",
        config.namespaces.len(),
        if config.namespaces.len() == 1 {
            ""
        } else {
            "s"
        }
    );

    create_uploads_directory(&config)?;

    let bind_address = format!("{}:{}", config.web_server.host, config.web_server.port);

    let server = HttpServer::new({
        let config_closure = config.clone();
        move || {
            let mut app = App::new()
                .app_data(config_closure.clone())
                .app_data(
                    MultipartFormConfig::default()
                        .total_limit(config_closure.storage.max_file_size_bytes),
                )
                .app_data(TempFileConfig::default().directory(config_closure.get_temp_path()))
                .route("/", web::get().to(index_redirect))
                .route("/upload", web::put().to(upload::upload));

            // attach a static file router for all namespaces
            for namespace in &config_closure.namespaces {
                let mut files = Files::new(
                    namespace.0,
                    namespace.1.get_path(&config_closure).to_str().unwrap_or_else(|| panic!("should be able to convert path of namespace {} to str",
                            namespace.0)),
                );

                if namespace.1.file_listing.show {
                    files = files.show_files_listing();
                    if namespace.1.file_listing.use_fancy_renderer {
                        files = files.files_listing_renderer(directory_listing);
                    }
                }

                app = app.service(files);
            }

            app.service(Files::new(
                "/",
                config_closure
                    .get_uploads_path()
                    .join(&config_closure.storage.default_namespace_fs_path),
            ))
        }
    })
    .bind(&bind_address)?;

    info!("Server listening on http://{}/", &bind_address);
    info!("Configured public URL: {}", &config.web_server.listen_url);

    server.run().await
}
