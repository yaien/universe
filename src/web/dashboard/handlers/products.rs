use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, Responder};
use anyhow::Context;
use maud::{Markup, html};
use serde::Deserialize;

use crate::app::{App, Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::views;
use crate::web::dashboard::views::layout::Content;
use crate::web::errors::WebError;

#[derive(Deserialize, Default)]
pub struct ProductsQuery {
    pub fragment: Option<ProductsFragment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductsFragment {
    Create,
}

pub async fn get_index(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    query: Query<ProductsQuery>,
    req: HttpRequest,
) -> Result<Markup, WebError> {
    let mut query = query.into_inner();
    if !req.headers().contains_key("HX-Request") {
        query = ProductsQuery::default();
    }

    match query.fragment {
        Some(ProductsFragment::Create) => Ok(views::products::create_modal()),
        _ => {
            let products = app.store.products.get_by_organization_id(&org.id).await?;

            Ok(views::layout::layout(&Content {
                title: "Productos",
                path: req.path(),
                org: &org,
                role: &role,
                content: views::products::product_list(products),
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct CreateProductForm {
    pub name: String,
}

pub async fn exec_index_actions(
    app: Data<App>,
    org: ReqData<Organization>,
    form: Form<CreateProductForm>,
) -> Result<impl Responder, WebError> {
    let product_id = app.store.products.create(&org.id, &form.name).await?;
    let product_url = format!("/dashboard/products/{}", product_id);
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Location", product_url))
        .finish())
}

#[derive(Deserialize)]
pub struct ProductDetailQuery {
    pub presentation_id: Option<Id>,
    pub content_id: Option<Id>,
}

pub async fn get_details(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
    product_id: Path<Id>,
    query: Query<ProductDetailQuery>,
) -> Result<Markup, WebError> {
    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = query
        .presentation_id
        .and_then(|id| product.presentations.iter().find(|p| p.id == id))
        .or(product.presentations.first());

    let content = query
        .content_id
        .and_then(|id| presentation.map(|p| p.contents.iter().find(|c| c.id == id)))
        .flatten();

    match req.headers().get("HX-Target").and_then(|h| h.to_str().ok()) {
        Some("article#pictures") => Ok(html! {
            (views::products::pictures(&product, &presentation, &content))
        }),
        Some("article#presentations") => Ok(html! {
            (views::products::presentations(&product, &presentation))
            (views::products::pictures_partial(&product, &presentation, &None))
        }),
        _ => Ok(views::layout::layout(&Content {
            title: "Product Details",
            path: req.path(),
            org: &org,
            role: &role,
            content: views::products::product_detail(&product, &presentation),
        })),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum ProductDetailAction {
    CreatePresentation,
    DeleteContent {
        presentation_id: String,
        content_id: String,
    },
    SortContent {
        presentation_id: Id,
        toggled_content_id: Id,
        toggled_new_number: i64,
    },
}

pub async fn exec_detail_actions(
    app: Data<App>,
    org: ReqData<Organization>,
    product_id: Path<Id>,
    action: Form<ProductDetailAction>,
) -> Result<Markup, WebError> {
    use ProductDetailAction::*;
    match action.into_inner() {
        CreatePresentation => {
            let presentation = app.store.presentations.create(&org.id, &product_id).await?;
            let product = app
                .store
                .products
                .get_one_by_organization_id_and_id(&org.id, &product_id)
                .await?;
            Ok(views::products::presentations(
                &product,
                &Some(&presentation),
            ))
        }
        DeleteContent {
            presentation_id,
            content_id,
        } => {
            let presentation_id: Id = presentation_id.parse().map_err(|e| {
                WebError::Status(
                    StatusCode::BAD_REQUEST,
                    format!("invalid presentation id: {}", e),
                )
            })?;

            let content_id: Id = content_id.parse().map_err(|e| {
                WebError::Status(
                    StatusCode::BAD_REQUEST,
                    format!("invalid content id: {}", e),
                )
            })?;

            app.store
                .contents
                .delete(&org.id, &product_id, &presentation_id, &content_id)
                .await?;

            let product = app
                .store
                .products
                .get_one_by_organization_id_and_id(&org.id, &product_id)
                .await?;

            let presentation = product
                .presentations
                .iter()
                .find(|p| p.id == presentation_id);

            Ok(views::products::pictures(&product, &presentation, &None))
        }
        SortContent {
            presentation_id,
            toggled_content_id,
            toggled_new_number,
        } => Ok(html!()),
    }
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    pub files: Vec<TempFile>,
}

pub async fn upload_content(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id) = path.into_inner();

    app.store
        .contents
        .upload_many(&org.id, &product_id, &presentation_id, form.files)
        .await?;

    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = product
        .presentations
        .iter()
        .find(|p| p.id == presentation_id);

    let content = presentation.and_then(|p| p.contents.last());

    Ok(views::products::pictures(&product, &presentation, &content))
}
