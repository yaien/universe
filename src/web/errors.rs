use actix_web::error::ResponseError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, HttpResponseBuilder};
use derive_more::Display;

#[derive(Debug, Display)]
#[display("Redirect to {}", 0)]
pub struct RedirectError(pub String);

impl From<&str> for RedirectError {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

impl ResponseError for RedirectError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::TemporaryRedirect()
            .insert_header((header::LOCATION, &self.0[..]))
            .finish()
    }
}

#[derive(Debug, Display)]
#[display("Status {}: {}", 0, 1)]
pub struct StatusError(pub StatusCode, pub String);

impl From<(StatusCode, &str)> for StatusError {
    fn from(value: (StatusCode, &str)) -> Self {
        Self(value.0, String::from(value.1))
    }
}

impl ResponseError for StatusError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.0).body(self.1.clone())
    }
}
