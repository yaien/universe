use axum::{
    Extension,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Utc};
use oauth2::PkceCodeVerifier;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, TokenResponse};
use serde::Deserialize;
use url::Url;

use crate::app::{self, GoogleUserInfo, OAuthAccountInfo, OAuthState, Organization};
use crate::infra::Monolith;

const SESSION_COOKIE_NAME: &'static str = "session";

pub async fn index(Extension(org): Extension<Organization>) -> String {
    format!("Hello, World! {}", org.title)
}

pub async fn login(
    State(mono): State<Monolith>,
    Extension(org): Extension<Organization>,
) -> (impl IntoResponse) {
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

    let Ok(mut conn) = mono.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if app::create_oauth_state(&mut conn, state).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    Redirect::to(redirect_url.as_str()).into_response()
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

pub async fn callback(
    State(mono): State<Monolith>,
    Extension(org): Extension<Organization>,
    jar: PrivateCookieJar,
    query: Query<OAuthCallbackQuery>,
) -> Response {
    let Ok(mut conn) = mono.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok(oauth_state) = app::get_oauth_state_by_csrf_token(&mut conn, &query.state).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if oauth_state.hostname != org.hostname {
        match app::get_organization_by_host(&mut conn, &oauth_state.hostname).await {
            Ok(org) => {
                let mut url = Url::parse(&org.url).unwrap();
                url.set_path("/oauth/google/callback");
                url.set_query(Some(&format!("code={}&state={}", query.code, query.state)));
                return Redirect::to(url.as_str()).into_response();
            }
            Err(_) => {
                return StatusCode::UNAUTHORIZED.into_response();
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
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if app::delete_oauth_state_by_csrf_token(&mut conn, &oauth_state.csrf_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let Ok(response) = mono
        .http_client
        .get(mono.config.google_user_info_url.clone())
        .bearer_auth(res.access_token().secret())
        .send()
        .await
        .and_then(|r| r.error_for_status())
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let user_info = match response.json::<GoogleUserInfo>().await {
        Ok(user_info) => user_info,
        Err(e) => {
            eprintln!("failed to parse google user info: {}", e);
            return StatusCode::UNAUTHORIZED.into_response();
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

    let Ok(user) = app::sync_oauth_account(&mut conn, account_info).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let cookie = Cookie::build((SESSION_COOKIE_NAME, user.id.to_string()))
        .secure(mono.config.session_secure)
        .http_only(true)
        .same_site(SameSite::Lax);

    let private = jar.add(cookie);

    (private, Redirect::temporary("/dashboard")).into_response()
}
