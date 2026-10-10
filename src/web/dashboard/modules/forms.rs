mod form_handlers;
mod form_views;

use actix_web::web::ServiceConfig;
use form_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(get_forms)
        .service(get_form)
        .service(create_form)
        .service(update_form)
        .service(create_field)
        .service(update_field)
        .service(delete_field)
        .service(get_form_submissions);
}
