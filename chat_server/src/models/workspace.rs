use super::Workspace;
use crate::error::AppError;
use chat_core::ChatUser;
use sqlx::PgPool;

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

    pub async fn get_by_id(id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn get_by_name(name: &str, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(id: u64, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces
            SET owner_id=$1
            WHERE id=$2 AND owner_id = 0
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn list_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT u.id, u.fullname, u.email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use anyhow::Result;
    use sqlx::PgPool;
    use tokio::fs;

    use crate::models::Workspace;

    #[tokio::test]
    async fn test_workspace_should_create() -> Result<()> {
        let current_dir = env::current_dir().expect("Failed to get current dir");
        println!("Current directory: {}", current_dir.display());

        let pool =
            PgPool::connect("postgres://db_manager:super_admin@localhost:5432/chat_test").await?;

        // let create_table_sql =
        //     fs::read_to_string("../migrations/20240706160746_initial.sql").await?;

        // println!("{:?}", pool);
        // println!("{}", create_table_sql);

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id bigserial PRIMARY KEY,
                name VARCHAR(32) NOT NULL UNIQUE,
                owner_id bigint NOT NULL,
                created_at timestamptz DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&pool)
        .await?;

        println!("start create workspace...");

        let ws = Workspace::create("test-ws2", 2, &pool).await?;
        println!("created workspace: {:?}", ws);
        assert_eq!(ws.name, "test-ws2");
        assert_eq!(ws.owner_id, 2);

        let ws1 = Workspace::get_by_id(ws.id as u64, &pool).await?;
        println!("get workspace: {:?}", ws1);
        assert_eq!(ws, ws1);
        Ok(())
    }
}
