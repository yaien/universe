use actix_web::get;
use actix_web::web::ReqData;

use crate::app::Organization;
use crate::app::User;
use std::ops::Deref;

#[get("/")]
pub async fn index(org: ReqData<Organization>, user: ReqData<Option<User>>) -> String {
    match user.deref() {
        Some(user) => format!("Hello, World! {}", user.name),
        None => format!("Hello, World! {}", org.title),
    }
}
