use std::ops::Deref;

use actix_session::Session;
use actix_web::http::header;
use actix_web::web::{Data, Query, ReqData};
use actix_web::{HttpResponse, Responder, get};
use chrono::Utc;
use oauth2::PkceCodeVerifier;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, TokenResponse};
use serde::Deserialize;
use url::Url;

use crate::app::{App, GoogleUserInfo, OAuthAccountInfo, OAuthState, Organization, User};
use crate::infra::Monolith;

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

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

pub async fn callback(
    mono: Data<Monolith>,
    app: Data<App>,
    org: ReqData<Organization>,
    session: Session,
    query: Query<OAuthCallbackQuery>,
) -> impl Responder {
    let Ok(oauth_state) = app.auth.get_oauth_state_by_csrf_token(&query.state).await else {
        return HttpResponse::Unauthorized().body("unauthorized");
    };

    if oauth_state.hostname != org.hostname {
        match app
            .organizations
            .get_one_by_host(&oauth_state.hostname)
            .await
        {
            Ok(org) => {
                let mut url = Url::parse(&org.url).unwrap();
                url.set_path(&mono.config.google_callback_path);
                url.set_query(Some(&format!("code={}&state={}", query.code, query.state)));
                return HttpResponse::TemporaryRedirect()
                    .insert_header((header::LOCATION, url.as_str()))
                    .finish();
            }
            Err(_) => {
                return HttpResponse::Unauthorized().body("unauthorized");
            }
        }
    }

    let pkce_verifier = PkceCodeVerifier::new(oauth_state.pkce_verifier);

    let authorization_code = AuthorizationCode::new(query.code.clone());

    let Ok(res) = mono
        .oauth2_google_client
        .exchange_code(authorization_code)
        .set_pkce_verifier(pkce_verifier)
        .request_async(&mono.oauth2_client)
        .await
    else {
        return HttpResponse::Unauthorized().body("unauthorized");
    };

    if app
        .auth
        .delete_oauth_state_by_csrf_token(&oauth_state.csrf_token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("failed to delete oauth state");
    }

    let Ok(response) = mono
        .http_client
        .get(mono.config.google_user_info_url.clone())
        .bearer_auth(res.access_token().secret())
        .send()
        .await
        .and_then(|r| r.error_for_status())
    else {
        return HttpResponse::Unauthorized().body("unauthorized");
    };

    let user_info = match response.json::<GoogleUserInfo>().await {
        Ok(user_info) => user_info,
        Err(e) => {
            eprintln!("failed to parse google user info: {}", e);
            return HttpResponse::Unauthorized().body("unauthorized");
        }
    };

    let account_info = OAuthAccountInfo {
        email: user_info.email,
        name: user_info.name,
        provider_user_id: user_info.sub,
        provider_name: "google".to_string(),
        access_token: res.access_token().secret().clone(),
        refresh_token: res.refresh_token().map(|t| t.secret().clone()),
        expires_at: res.expires_in().map(|duration| Utc::now() + duration),
    };

    let user = match app.auth.sync_oauth_account(account_info).await {
        Ok(user) => user,
        Err(e) => {
            eprintln!("failed to sync oauth account: {}", e);
            return HttpResponse::InternalServerError().body("failed to sync oauth account");
        }
    };

    session.insert("user_id", user.id.to_string()).unwrap();

    HttpResponse::TemporaryRedirect()
        .insert_header((header::LOCATION, "/"))
        .finish()
}
