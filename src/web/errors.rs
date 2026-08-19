use actix_web::error::ResponseError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, HttpResponseBuilder};
use log::error;
use thiserror::Error;

use crate::app;
use crate::app::Error::Message;

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

impl From<&app::Error> for WebError {
    fn from(err: &app::Error) -> Self {
        match err {
            app::Error::Message(message) => {
                WebError::Status(StatusCode::BAD_REQUEST, message.clone())
            }
            app::Error::Sqlx(err) => {
                WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        }
    }
}

impl ResponseError for app::Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            app::Error::Message(msg) => HttpResponse::BadRequest().body(msg.clone()),
            app::Error::Sqlx(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
}
