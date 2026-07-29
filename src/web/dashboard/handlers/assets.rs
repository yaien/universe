use actix_web::http::header;
use actix_web::web::Path;
use actix_web::{HttpRequest, HttpResponse, Responder};

use crate::web::dashboard::assets::Asset;

pub async fn assets(req: HttpRequest, file_path: Path<String>) -> impl Responder {
    if file_path.is_empty() {
        return HttpResponse::NotFound().finish();
    }

    let Some(content) = Asset::get(&file_path) else {
        return HttpResponse::NotFound().finish();
    };

    let etag_value = hex::encode(content.metadata.sha256_hash());

    let if_none_match = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|h| h.to_str().ok());

    if if_none_match == Some(&etag_value) {
        return HttpResponse::NotModified().finish();
    }

    let mime_type = mime_guess::from_path(file_path.into_inner()).first_or_octet_stream();

    HttpResponse::Ok()
        .content_type(mime_type)
        .insert_header((
            header::CACHE_CONTROL,
            "public, max-age=3600, must-revalidate",
        ))
        .insert_header((header::ETAG, etag_value))
        .body(content.data.into_owned())
}
