use actix_web::error::ResponseError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, HttpResponseBuilder};
use thiserror::Error;

use crate::app::AppError;

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

impl From<AppError> for WebError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Message(message) => {
                WebError::Status(StatusCode::BAD_REQUEST, message.clone())
            }
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
