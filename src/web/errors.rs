use actix_web::error::ResponseError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, HttpResponseBuilder};
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebError {
    #[error("redirect to {0}")]
    Redirect(String),

    #[error("status {0}: {1}")]
    Status(StatusCode, String),
}

impl ResponseError for WebError {
    fn error_response(&self) -> HttpResponse {
        use WebError::*;

        match self {
            // Redirect to a given url
            Redirect(url) => HttpResponse::TemporaryRedirect()
                .insert_header((header::LOCATION, &url[..]))
                .finish(),

            // Return a status code and message
            Status(code, msg) => HttpResponseBuilder::new(*code).body(msg.clone()),
        }
    }
}
