use actix_web::Error as ActixError;
use actix_web::error::ResponseError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, HttpResponseBuilder};
use thiserror::Error;

use crate::app::AppError;
use crate::web::dashboard::modules::base::{Variant, toast};

#[derive(Debug, Error)]
pub enum WebError {
    #[error("redirect to {0}")]
    Redirect(String),

    #[error("status {0}: {1}")]
    Status(StatusCode, String),

    #[error("{0}")]
    Message(String),

    #[error("{0}")]
    Actix(ActixError),
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
            Status(code, msg) => HttpResponseBuilder::new(*code).body(toast(&msg, Variant::Danger)),

            // Return a message
            Message(msg) => HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .body(toast(&msg, Variant::Warning)),

            // Return an actix error
            Actix(err) => err.error_response(),
        }
    }
}

impl From<AppError> for WebError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Message(message) => WebError::Message(message),
            AppError::Sqlx(err) => {
                WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AppError::Any(err) => {
                WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        }
    }
}

impl From<sqlx::Error> for WebError {
    fn from(err: sqlx::Error) -> Self {
        WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl From<anyhow::Error> for WebError {
    fn from(err: anyhow::Error) -> Self {
        WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl From<(StatusCode, &str)> for WebError {
    fn from((code, msg): (StatusCode, &str)) -> Self {
        WebError::Status(code, msg.to_string())
    }
}

impl From<(StatusCode, String)> for WebError {
    fn from((code, msg): (StatusCode, String)) -> Self {
        WebError::Status(code, msg)
    }
}

impl From<ActixError> for WebError {
    fn from(err: ActixError) -> Self {
        WebError::Actix(err)
    }
}
