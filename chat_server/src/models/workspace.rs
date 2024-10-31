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

    pub async fn get_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
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
            SELECT users.id, users.fullname, users.email
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
    use crate::{
        models::{user::CreateUser, Workspace},
        User,
    };
    use anyhow::Result;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_workspace_should_create() -> Result<()> {
        let pool =
            PgPool::connect("postgres://db_manager:super_admin@localhost:5432/chat_test").await?;

        sqlx::migrate!("../migrations").run(&pool).await?;

        println!("start create workspace...");

        let ws = Workspace::create("test-ws2", 2, &pool).await?;
        println!("created workspace: {:?}", ws);
        assert_eq!(ws.name, "test-ws2");
        assert_eq!(ws.owner_id, 2);

        let ws1 = Workspace::get_by_id(ws.id as u64, &pool).await?;
        println!("get workspace: {:?}", ws1);
        assert_eq!(ws, ws1);

        sqlx::query(r#"TRUNCATE TABLE workspaces;"#)
            .execute(&pool)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_workspace_list_all_chat_users() -> Result<()> {
        let pool =
            PgPool::connect("postgres://db_manager:super_admin8801@localhost:5432/chat_test")
                .await?;

        sqlx::migrate!("../migrations").run(&pool).await?;

        println!("start create workspace...");

        let ws = Workspace::create("test-user-workspace", 0, &pool).await?;
        println!("created workspace: {:?}", ws);

        let create_user1 = &CreateUser {
            fullname: "Bob".to_string(),
            email: "bob@acme.com".to_string(),
            workspace: "test-user-workspace".to_string(),
            password: "test-passAbc8".to_string(),
        };
        let user1 = User::create_user(&pool, create_user1).await?;

        println!("created user1: {:?}", user1);

        let create_user2 = &CreateUser {
            fullname: "Alice".to_string(),
            email: "alice@acme.com".to_string(),
            workspace: "test-user-workspace".to_string(),
            password: "test-passAbc9".to_string(),
        };
        let user2 = User::create_user(&pool, create_user2).await?;
        println!("created user2: {:?}", user2);

        let users = Workspace::list_all_chat_users(ws.id as u64, &pool).await?;
        println!("get users: {:?}", users);

        assert_eq!(users.len(), 2);

        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);
        assert_eq!(users[0].fullname, user1.fullname);
        assert_eq!(users[1].fullname, user2.fullname);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS chat_type;"#)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
