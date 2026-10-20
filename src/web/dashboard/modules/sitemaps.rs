mod sitemaps_handlers;
mod sitemaps_views;

use actix_web::web::ServiceConfig;
use sitemaps_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(pages)
        .service(get_preview)
        .service(create_page)
        .service(create_layout)
        .service(publish)
        .service(update_page)
        .service(update_layout)
        .service(update_email)
        .service(delete_page)
        .service(delete_layout)
        .service(delete_sitemap)
        .service(sync_branch)
        .service(upload_file)
        .service(update_file)
        .service(delete_file)
        .service(create_font)
        .service(update_font)
        .service(create_color)
        .service(update_color)
        .service(delete_color)
        .service(update_html)
        .service(update_css)
        .service(update_js)
        .service(update_og_image)
        .service(update_favicon)
        .service(create_action)
        .service(update_action)
        .service(get_action_form_fieldset)
        .service(call_action)
        .service(create_form)
        .service(update_form)
        .service(delete_form)
        .service(get_delete_form)
        .service(get_create_form_field_form)
        .service(get_edit_form_field_form)
        .service(create_form_field)
        .service(update_form_field)
        .service(delete_form_field)
        .service(create_var);
}
