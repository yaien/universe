use actix_web::web::ReqData;
use actix_web::{HttpRequest, get};
use maud::{Markup, html};

use crate::app::auth::{Organization, Role};
use crate::web::dashboard::modules::base::{Content, page};

#[get("/empty")]
pub async fn empty() -> Markup {
    html!()
}

#[get("")]
pub async fn home(org: ReqData<Organization>, role: ReqData<Role>, req: HttpRequest) -> Markup {
    page(&Content {
        title: "Home",
        path: req.path(),
        org: &org.into_inner(),
        role: &role.into_inner(),
        content: html!(),
    })
}
