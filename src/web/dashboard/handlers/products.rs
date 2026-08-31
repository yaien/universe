use actix_web::HttpRequest;
use actix_web::web::{Data, ReqData};
use maud::Markup;

use crate::app::{App, Organization, Role};
use crate::web::dashboard::views;
use crate::web::dashboard::views::layout::Content;
use crate::web::errors::WebError;

pub async fn products(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
) -> Result<Markup, WebError> {
    let products = app.products.get_by_organization_id(&org.id).await?;

    Ok(views::layout::layout(&Content {
        title: "Productos",
        path: req.path(),
        org: &org,
        role: &role,
        content: views::products::product_list(products),
    }))
}
