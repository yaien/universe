use crate::infra::{DbPool, Id};

use sqlx::prelude::FromRow;

#[derive(FromRow, Clone)]
pub struct Role {
    pub user_name: String,
    pub user_email: String,
}

pub struct Roles {
    pool: DbPool,
}

impl Roles {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_one_by_org_id_and_user_id(
        &self,
        org_id: &Id,
        user_id: &Id,
    ) -> Result<Role, sqlx::Error> {
        let query = r#"
            select r.id, r.user_id, u.name as user_name, u.email as user_email, r.organization_id, r.created_at
            from roles r
            join users u on r.user_id = u.id
            where r.organization_id = $1 and r.user_id = $2
        "#;

        sqlx::query_as::<_, Role>(query)
            .bind(org_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_org_id(&self, org_id: &Id) -> Result<Vec<Role>, sqlx::Error> {
        let query = r#"
            select r.id, r.user_id, u.name as user_name, u.email as user_email, r.organization_id, r.created_at
            from roles r
            join users u on r.user_id = u.id
            where r.organization_id = $1
        "#;

        sqlx::query_as::<_, Role>(query)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, migrate};

    use super::*;

    #[sqlx::test]
    async fn test_get_by_org_id() -> Result<(), sqlx::Error> {
        let pool = SqlitePool::connect(":memory:").await?;
        migrate!().run(&pool).await?;
        let roles = Roles::new(pool.clone());
        roles.get_by_org_id(&1).await?;
        Ok(())
    }
}
