use actix_web::cookie::Key;
use dotenv::dotenv;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenUrl};
use std::{
    env::{self, VarError},
    net::SocketAddr,
};
use url::Url;

pub struct Config {
    pub database_url: Url,
    pub server_addr: SocketAddr,
    pub server_url: Url,
    pub server_tls: Option<ServerTlsConfig>,
    pub session_secure: bool,
    pub session_key: Key,
    pub google_client_id: ClientId,
    pub google_client_secret: ClientSecret,
    pub google_auth_url: AuthUrl,
    pub google_token_url: TokenUrl,
    pub google_user_info_url: Url,
    pub google_scopes: Vec<Scope>,
    pub encryption_key: String,
}

#[derive(Debug, Clone)]
pub struct ServerTlsConfig {
    pub cert_file_path: String,
    pub key_file_path: String,
}

impl Config {
    pub fn new_from_env() -> Self {
        dotenv().ok();

        Config {
            database_url: env::var("DATABASE_URL")
                .map(|s| s.parse().expect("invalid database url"))
                .expect("missing databae url"),

            server_addr: env::var("SERVER_ADDR")
                .unwrap_or("0.0.0.0:3000".to_string())
                .parse()
                .expect("invalid server addr"),

            server_url: env::var("SERVER_URL")
                .map(|s| s.parse().expect("invalid server url"))
                .expect("missing server url"),

            server_tls: {
                let enabled = env::var("SERVER_TLS")
                    .and_then(|s| s.parse().map_err(|_| VarError::NotPresent))
                    .unwrap_or(false);

                if enabled {
                    Some(ServerTlsConfig {
                        cert_file_path: env::var("SERVER_CERT_FILE_PATH")
                            .map(|s| s.into())
                            .expect("missing server cert file path"),

                        key_file_path: env::var("SERVER_KEY_FILE_PATH")
                            .map(|s| s.into())
                            .expect("missing server key file path"),
                    })
                } else {
                    None
                }
            },

            session_secure: env::var("SESSION_SECURE")
                .map(|s| s.parse().expect("invalid session secure"))
                .unwrap_or(false),

            session_key: {
                let key = env::var("SESSION_KEY")
                    .and_then(|x| hex::decode(x).map_err(|_| VarError::NotPresent))
                    .and_then(|x| Key::try_from(&x[..]).map_err(|_| VarError::NotPresent));

                match key {
                    Ok(key) => key,
                    Err(_) => {
                        let key = Key::generate();
                        let encoded = hex::encode(key.master());
                        panic!("invalid session key var: set SESSION_KEY={}", encoded)
                    }
                }
            },

            google_client_id: env::var("GOOGLE_CLIENT_ID")
                .map(|s| ClientId::new(s))
                .expect("missing google client id"),

            google_client_secret: ClientSecret::new(
                env::var("GOOGLE_CLIENT_SECRET").expect("missing google client secret"),
            ),

            google_auth_url: AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())
                .expect("invalid google auth url"),

            google_token_url: TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .expect("invalid google token url"),

            google_user_info_url: Url::parse("https://www.googleapis.com/oauth2/v3/userinfo")
                .expect("invalid google user info url"),

            google_scopes: vec![
                Scope::new("openid".to_string()),
                Scope::new("profile".to_string()),
                Scope::new("email".to_string()),
            ],

            encryption_key: env::var("ENCRYPTION_KEY").unwrap_or_default(),
        }
    }
}
