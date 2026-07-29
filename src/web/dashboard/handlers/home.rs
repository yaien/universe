use actix_web::HttpRequest;
use actix_web::web::ReqData;
use maud::{Markup, html};

use crate::app::{Organization, Role};
use crate::web::dashboard::views;

pub async fn home(org: ReqData<Organization>, role: ReqData<Role>, req: HttpRequest) -> Markup {
    views::layout::layout(&views::layout::Content {
        title: "Home",
        path: req.path(),
        org: &org.into_inner(),
        role: &role.into_inner(),
        content: html!(),
    })
}
