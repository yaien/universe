use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::http::header::ContentDisposition;
use actix_web::web::{Data, Path, Query, ReqData};
use actix_web::{Error, HttpResponse};
use maud::Markup;
use mime::{APPLICATION_OCTET_STREAM, Mime};
use serde::Deserialize;

use crate::app::{App, Organization, RegistryContext, RenderMode, RenderPageOptions, render_page};
use crate::app::{Branch, User};
use crate::web::errors::WebError;

use std::str::FromStr;
use std::sync::Arc;

pub async fn get_index(
    app: Data<App>,
    org: ReqData<Organization>,
    user: ReqData<Option<User>>,
    path: Path<String>,
) -> Result<Markup, WebError> {
    let sitemap = app
        .sitemaps
        .get_one_by_branch(&org.id, Branch::MAIN)
        .await?;

    let page = app.pages.get_by_path(&sitemap.id, &path).await?;

    let layout = match page.layout_id {
        Some(layout_id) => Some(app.layouts.get_by_id(&sitemap.id, &layout_id).await?),
        None => None,
    };

    let fonts = app.fonts.get_by_sitemap_id(&sitemap.id).await?;

    let mode = RenderMode::External;

    let ctx = RegistryContext {
        app: app.into_inner(),
        org: Arc::new(org.into_inner()),
        user: Arc::new(user.into_inner()),
    };

    let content = render_page(RenderPageOptions {
        ctx,
        page,
        layout,
        sitemap,
        fonts,
        mode,
    })?;

    Ok(content)
}

pub async fn get_bundled_css(
    app: Data<App>,
    org: ReqData<Organization>,
) -> Result<HttpResponse, WebError> {
    let css = app.sitemaps.get_bundled_css(&org.id).await?;
    Ok(HttpResponse::Ok().content_type("text/css").body(css))
}

pub async fn get_favicon(app: Data<App>, org: ReqData<Organization>) -> Result<NamedFile, Error> {
    let file_id = app
        .sitemaps
        .get_favicon_file_id(&org.id)
        .await
        .map_err(|err| WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let Some(file_id) = file_id else {
        return Err(WebError::Status(
            StatusCode::NOT_FOUND,
            "favicon not found".to_string(),
        ))?;
    };

    let mut file = app
        .files
        .get_one_by_organization_id_and_id(&org.id, &file_id)
        .await
        .map_err(|err| {
            WebError::Status(StatusCode::NOT_FOUND, format!("favicon not found: {}", err))
        })?;

    let (path, format) = app
        .files
        .get_path_and_format(&mut file, &0)
        .await
        .map_err(|err| {
            WebError::Status(StatusCode::NOT_FOUND, format!("favicon not found: {}", err))
        })?;

    let named = NamedFile::open(path)?
        .set_content_type(Mime::from_str(&format.content_type).unwrap_or(APPLICATION_OCTET_STREAM))
        .set_content_disposition(ContentDisposition::attachment(file.name));

    Ok(named)
}

pub async fn get_bundled_js(
    app: Data<App>,
    org: ReqData<Organization>,
) -> Result<HttpResponse, WebError> {
    let js = app.sitemaps.get_bundled_js(&org.id).await?;
    Ok(HttpResponse::Ok().content_type("text/javascript").body(js))
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
        .map_err(|e| WebError::Status(StatusCode::NOT_FOUND, e.to_string()))?;

    let variant = query.variant.unwrap_or(0);

    let (path, format) = app
        .files
        .get_path_and_format(&mut file, &variant)
        .await
        .map_err(|e| WebError::Status(StatusCode::NOT_FOUND, e.to_string()))?;

    let named = NamedFile::open(path)?
        .set_content_type(Mime::from_str(&format.content_type).unwrap_or(APPLICATION_OCTET_STREAM))
        .set_content_disposition(ContentDisposition::attachment(file.name));

    Ok(named)
}
