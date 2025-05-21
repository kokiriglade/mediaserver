use actix_web::{HttpResponse, http::header, web::Data};

use crate::config::Config;

pub async fn index_redirect(cfg: Data<Config>) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .insert_header((
            header::LOCATION,
            cfg.web_server.redirect_index_to.to_string(),
        ))
        .finish()
}
