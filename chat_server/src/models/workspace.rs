use super::Workspace;
use crate::{error::AppError, AppState};
use chat_core::ChatUser;

impl AppState {
    pub async fn create_workspace(&self, name: &str, owner_id: i64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            "INSERT INTO workspaces (name, owner_id)
            VALUES ($1,$2) RETURNING id,name,owner_id, created_at",
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn get_workspace_by_id(&self, id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn get_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_wokrspce_owner(
        &self,
        id: u64,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
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
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn list_all_chat_users(&self, id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT users.id, users.fullname, users.email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{AuthConfig, ServerConfig},
        models::user::CreateUser,
        AppConfig, AppState,
    };
    use anyhow::Result;
    use tokio::fs;

    #[tokio::test]
    async fn test_workspace_should_create() -> Result<()> {
        let p_key = fs::read_to_string("./fixture/private.pem").await?;
        let d_key = fs::read_to_string("./fixture/public.pem").await?;

        let config = AppConfig {
            server: ServerConfig {
                port: 8088,
                db_url: "postgres://db_manager:super_admin8801@localhost:5432/chat_test"
                    .to_string(),
                base_dir: "/tmp/chat".to_string(),
            },
            auth: AuthConfig {
                private_key: p_key,
                public_key: d_key,
            },
        };

        let app_state = AppState::try_new(config).await?;
        let pool = app_state.pool.clone();

        println!("start create workspace...");

        let ws = app_state.create_workspace("test-ws2", 2).await?;
        println!("created workspace: {:?}", ws);
        assert_eq!(ws.name, "test-ws2");
        assert_eq!(ws.owner_id, 2);

        let ws1 = app_state.get_workspace_by_id(ws.id as u64).await?;
        println!("get workspace: {:?}", ws1);
        assert_eq!(ws, ws1);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&pool)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_workspace_list_all_chat_users() -> Result<()> {
        let p_key = fs::read_to_string("./fixture/private.pem").await?;
        let d_key = fs::read_to_string("./fixture/public.pem").await?;

        let config = AppConfig {
            server: ServerConfig {
                port: 8088,
                db_url: "postgres://db_manager:super_admin8801@localhost:5432/chat_test"
                    .to_string(),
                base_dir: "/tmp/chat".to_string(),
            },
            auth: AuthConfig {
                private_key: p_key,
                public_key: d_key,
            },
        };

        let app_state = AppState::try_new(config).await?;
        let pool = app_state.pool.clone();

        println!("start create workspace...");

        let ws = app_state.create_workspace("test-user-workspace", 0).await?;
        println!("created workspace: {:?}", ws);

        let create_user1 = &CreateUser {
            fullname: "Bob".to_string(),
            email: "bob@acme.com".to_string(),
            workspace: "test-user-workspace".to_string(),
            password: "test-passAbc8".to_string(),
        };
        let user1 = app_state.create_user(create_user1).await?;

        println!("created user1: {:?}", user1);

        let create_user2 = &CreateUser {
            fullname: "Alice".to_string(),
            email: "alice@acme.com".to_string(),
            workspace: "test-user-workspace".to_string(),
            password: "test-passAbc9".to_string(),
        };
        let user2 = app_state.create_user(create_user2).await?;
        println!("created user2: {:?}", user2);

        let users = app_state.list_all_chat_users(ws.id as u64).await?;
        println!("get users: {:?}", users);

        assert_eq!(users.len(), 2);

        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);
        assert_eq!(users[0].fullname, user1.fullname);
        assert_eq!(users[1].fullname, user2.fullname);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
