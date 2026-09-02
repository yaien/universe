use actix_web::http::header;
use actix_web::web::{Data, Form, Header, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, Responder};
use maud::Markup;
use serde::Deserialize;

use crate::app::store::{Presentation, Product};
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

    match req.headers().get("HX-Target").and_then(|h| h.to_str().ok()) {
        Some("article#presentations") => {
            Ok(views::products::presentations(&product, &presentation))
        }
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
    }
}
