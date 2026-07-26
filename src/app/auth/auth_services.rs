use crate::{
    app::{OAuthAccount, OAuthAccountInfo, OAuthState, User},
    infra::{DBConnection, ID},
};
use anyhow::Result;

pub async fn create_oauth_state(conn: &mut DBConnection, state: OAuthState) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO oauth_states (csrf_token, pkce_verifier, created_at, expires_at, hostname, provider)
           VALUES ($1, $2, $3, $4, $5, $6)"#
    ).bind(&state.csrf_token)
     .bind(&state.pkce_verifier)
     .bind(&state.created_at)
     .bind(&state.expires_at)
     .bind(&state.hostname)
     .bind(&state.provider)
     .execute(conn).await?;
    Ok(())
}

pub async fn get_oauth_state_by_csrf_token(
    conn: &mut DBConnection,
    csrf_token: &str,
) -> Result<OAuthState> {
    sqlx::query_as::<_, OAuthState>(r#"SELECT * FROM oauth_states WHERE csrf_token = $1"#)
        .bind(csrf_token)
        .fetch_one(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn delete_oauth_state_by_csrf_token(
    conn: &mut DBConnection,
    csrf_token: &str,
) -> Result<()> {
    sqlx::query(r#"DELETE FROM oauth_states WHERE csrf_token = $1"#)
        .bind(csrf_token)
        .execute(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

pub async fn sync_oauth_account(conn: &mut DBConnection, info: OAuthAccountInfo) -> Result<User> {
    let account = sqlx::query_as::<_, OAuthAccount>(
        "select * from oauth_accounts where provider_user_id = $1 and provider_name = $2",
    )
    .bind(&info.provider_user_id)
    .bind(&info.provider_name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(account) = account {
        sqlx::query("update oauth_accounts set access_token = $1, refresh_token = $2, expires_at = $3 where id = $4")
                .bind(&info.access_token)
                .bind(&info.refresh_token)
                .bind(&info.expires_at)
                .bind(&account.id)
                .execute(&mut *conn)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

        let user = sqlx::query_as::<_, User>("select * from users where id = $1")
            .bind(&account.user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        return Ok(user);
    }

    let user = sqlx::query_as::<_, User>("select * from users where email = $1")
        .bind(&info.email)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let user = match user {
        Some(user) => {
            println!("user found");
            user
        }
        None => {
            sqlx::query_as::<_, User>("insert into users (email, name) values ($1, $2) returning *")
                .bind(&info.email)
                .bind(&info.name)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| anyhow::anyhow!(e))?
        }
    };

    sqlx::query(
        r#"insert into oauth_accounts
            (provider_user_id, provider_name, access_token, refresh_token, expires_at, user_id)
            values ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(&info.provider_user_id)
    .bind(&info.provider_name)
    .bind(&info.access_token)
    .bind(&info.refresh_token)
    .bind(&info.expires_at)
    .bind(&user.id)
    .execute(&mut *conn)
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    return Ok(user);
}

pub async fn get_user_by_id(conn: &mut DBConnection, user_id: &ID) -> Result<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id")
        .bind(&user_id)
        .fetch_one(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn test_sync_oauth_account() {
        let mut conn = SqliteConnection::connect(":memory:").await.unwrap();

        sqlx::migrate!().run(&mut conn).await.unwrap();

        let info = OAuthAccountInfo {
            provider_user_id: "id".to_string(),
            provider_name: "google".to_string(),
            access_token: "access_token".to_string(),
            refresh_token: None,
            expires_at: None,
            email: "test@example.com".to_string(),
            name: "test".to_string(),
        };

        let user = sync_oauth_account(&mut conn, info).await.unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "test");

        let info = OAuthAccountInfo {
            provider_user_id: "id".to_string(),
            provider_name: "google".to_string(),
            access_token: "access_token".to_string(),
            refresh_token: None,
            expires_at: None,
            email: "test@example.com".to_string(),
            name: "test".to_string(),
        };

        sync_oauth_account(&mut conn, info).await.unwrap();
    }
}
