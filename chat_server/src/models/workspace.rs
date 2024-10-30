use sqlx::PgPool;

use crate::error::AppError;

use super::Workspace;

impl Workspace {
    pub async fn create(name: &str, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            "INSERT INTO workspaces (name, owner_id)
            VALUES ($1,$2) RETURNING id,name,owner_id, created_at",
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }
}
