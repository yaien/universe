use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header;

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();

    HttpResponse::TemporaryRedirect()
        .insert_header((header::LOCATION, "/"))
        .finish()
}
