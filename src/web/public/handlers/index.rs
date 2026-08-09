use actix_files::NamedFile;
use actix_web::Error;
use actix_web::http::StatusCode;
use actix_web::http::header::ContentDisposition;
use actix_web::web::{Data, Path, Query, ReqData};
use mime::{APPLICATION_OCTET_STREAM, Mime};
use serde::Deserialize;

use crate::app::User;
use crate::app::{App, Organization};
use crate::web::errors::StatusError;

use std::ops::Deref;
use std::str::FromStr;

pub async fn get_index(org: ReqData<Organization>, user: ReqData<Option<User>>) -> String {
    match user.deref() {
        Some(user) => format!("Hello, World! {}", user.name),
        None => format!("Hello, World! {}", org.title),
    }
}

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    variant: Option<u32>,
}

pub async fn download_file(
    app: Data<App>,
    org: ReqData<Organization>,
    name: Path<String>,
    query: Query<FileQuery>,
) -> Result<NamedFile, Error> {
    let mut file = app
        .files
        .get_one_by_organization_id_and_name(&org.id, &name)
        .await
        .map_err(|e| StatusError(StatusCode::NOT_FOUND, e.to_string()))?;

    let variant = query.variant.unwrap_or(0);

    let (path, format) = app
        .files
        .get_path_and_format(&mut file, &variant)
        .await
        .map_err(|e| StatusError(StatusCode::NOT_FOUND, e.to_string()))?;

    let named = NamedFile::open(path)?
        .set_content_type(Mime::from_str(&format.content_type).unwrap_or(APPLICATION_OCTET_STREAM))
        .set_content_disposition(ContentDisposition::attachment(file.name));

    Ok(named)
}
