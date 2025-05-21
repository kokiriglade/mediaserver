use std::path::Path;

use crate::config::{Config, NamespaceDefinition};
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{HttpResponse, web::Data};
use log::error;
use serde::Serialize;
use url::Url;

/// Data we expect to receive during uploads.
#[derive(Debug, MultipartForm)]
pub struct UploadData {
    file: TempFile,
    namespace: Text<String>,
    auth_key: Text<String>,
}

/// The response payload. Contains either a URL or an error message.
#[derive(Serialize)]
pub struct ResponsePayload {
    #[serde(with = "url_serde")]
    link: Option<Url>,
    error: Option<String>,
}

impl ResponsePayload {
    pub fn of_link(url: Url) -> ResponsePayload {
        Self {
            link: Some(url),
            error: None,
        }
    }

    pub fn of_error(error_message: String) -> ResponsePayload {
        Self {
            link: None,
            error: Some(error_message),
        }
    }
}

/// The file upload endpoint.
pub async fn upload(
    cfg: Data<Config>,
    MultipartForm(form): MultipartForm<UploadData>,
) -> HttpResponse {
    let input_namespace = &form.namespace.0;
    let input_auth_key = &form.auth_key.0;

    // get the namespace definition and also authenticate
    let namespace = match NamespaceDefinition::auth(
        &cfg.namespaces,
        input_namespace,
        input_auth_key,
    ) {
        Some(ns) => ns,
        None => {
            return HttpResponse::Unauthorized().json(
                ResponsePayload::of_error("Failed to authenticate".to_string()),
            );
        }
    };

    let file_path =
        namespace.create_random_file_name(&cfg, get_file_extension(&form.file));

    if file_path.is_err() {
        error!(
            "Failed to create path for uploaded file: {:?}",
            file_path.err()
        );
        return HttpResponse::InternalServerError().json(
            ResponsePayload::of_error(
                "Failed to create path for uploaded file".to_string(),
            ),
        );
    }

    let file_path = file_path.unwrap();

    let persist = form.file.file.persist(&file_path);

    if persist.is_err() {
        error!(
            "Failed to parsist uploaded file: {}",
            persist.err().unwrap()
        );
        return HttpResponse::InternalServerError().json(
            ResponsePayload::of_error(
                "Failed to persist uploaded file".to_string(),
            ),
        );
    }

    HttpResponse::Ok().json(ResponsePayload::of_link(
        cfg.web_server
            .listen_url
            .join(format!("{}/", input_namespace).as_str())
            .expect("should be able to join with input_namespace")
            .join(
                file_path
                    .file_name()
                    .expect("should have a file name")
                    .to_str()
                    .expect("should be able to convert OsStr to str"),
            )
            .expect("should be able to join with file stem"),
    ))
}

/// Extracts file extension from a [`TempFile`].
fn get_file_extension(file: &TempFile) -> &str {
    let file_name = file.file_name.as_deref().unwrap_or("unnamed");

    Path::new(file_name)
        .extension()
        .and_then(|os| os.to_str())
        .unwrap_or("")
}
