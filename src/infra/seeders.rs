use std::pin::Pin;

use chrono::Utc;

use crate::infra::Monolith;
mod google_fonts;

pub struct Seeder {
    pub name: &'static str,
    pub seed: Pin<Box<dyn Fn(&Monolith) -> anyhow::Result<()>>>,
}

pub struct Seeders {}

macro_rules! register {
    ($module: ident, $mono: expr) => {
        async {
            let count: u8 = sqlx::query_scalar("select count(*) from seeds where name = $1")
                .bind($module::name())
                .fetch_one(&$mono.pool)
                .await?;

            if count == 0 {
                return Ok(());
            }

            let result = $module::run($mono).await;

            if result.is_err() {
                return result;
            }

            sqlx::query("insert into seeds(name, applied_at) values ($1, $2)")
                .bind($module::name())
                .bind(Utc::now())
                .execute(&$mono.pool)
                .await?;

            Ok(())
        }
    };
}

pub async fn run(mono: &Monolith) -> anyhow::Result<()> {
    register!(google_fonts, &mono).await?;

    Ok(())
}
