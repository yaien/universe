use actix_web::web::{Data, Path, ReqData};
use actix_web::{HttpRequest, get};
use maud::Markup;

use super::orders_views as views;
use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::modules::base::{Content, page};
use crate::web::errors::WebResult;

#[get("/orders")]
pub async fn order_list(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let orders = app.store.orders.get_by_organization_id(&org.id).await?;
    Ok(page(&Content {
        title: "Ordenes de Pago",
        path: req.path(),
        org: &org,
        role: &role,
        content: views::order_list(&orders),
    }))
}

#[get("/orders/{order_id}")]
pub async fn order_details(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
    order_id: Path<Id>,
) -> WebResult<Markup> {
    let order = app
        .store
        .orders
        .get_by_organization_id_and_id(&org.id, &order_id)
        .await?;
    Ok(page(&Content {
        title: "Detalles de Orden de Pago",
        path: req.path(),
        org: &org,
        role: &role,
        content: views::order_details(order),
    }))
}
