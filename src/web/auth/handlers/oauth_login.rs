use crate::app::{App, OAuthState, Organization};
use crate::infra::Monolith;
use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web::web::{Data, ReqData};
use oauth2::{CsrfToken, PkceCodeChallenge};

pub async fn login(
    mono: Data<Monolith>,
    app: Data<App>,
    org: ReqData<Organization>,
) -> HttpResponse {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (redirect_url, csrf_token) = mono
        .oauth2_google_client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .add_scopes(mono.config.google_scopes.clone())
        .url();

    let state = OAuthState {
        id: 0,
        csrf_token: csrf_token.into_secret(),
        pkce_verifier: pkce_verifier.into_secret(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        hostname: org.hostname.clone(),
        provider: "google".to_string(),
    };

    if app.auth.create_oauth_state(state).await.is_err() {
        return HttpResponse::InternalServerError().body("failed creating oauth state");
    };

    HttpResponse::TemporaryRedirect()
        .insert_header((header::LOCATION, redirect_url.as_str()))
        .finish()
}
