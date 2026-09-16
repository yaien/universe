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
        .service(update_favicon);
}
