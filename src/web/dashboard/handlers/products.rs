use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, Responder};
use maud::Markup;
use serde::Deserialize;

use crate::app::{App, Organization, Role};
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

pub async fn products(
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
            let products = app.products.get_by_organization_id(&org.id).await?;

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

pub async fn products_actions(
    app: Data<App>,
    org: ReqData<Organization>,
    form: Form<CreateProductForm>,
) -> Result<impl Responder, WebError> {
    let product_id = app.products.create(&org.id, &form.name).await?;
    let product_url = format!("/dashboard/products/{}", product_id);
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Location", product_url))
        .finish())
}
