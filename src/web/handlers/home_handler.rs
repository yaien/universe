use actix_web::web::ReqData;
use actix_web::{HttpRequest, get};
use maud::{Markup, html};

use crate::app::Organization;
use crate::web::views::dashboard_view;

#[get("/dashboard")]
pub async fn index(org: ReqData<Organization>, req: HttpRequest) -> Markup {
    dashboard_view::page("Home", req.path(), &org.into_inner(), html!())
}
