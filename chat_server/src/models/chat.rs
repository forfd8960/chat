use super::{Chat, ChatType};
use crate::{error::AppError, AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub public: bool,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub chat_type: Option<ChatType>,
    pub members: Vec<i64>,
}

impl AppState {
    pub async fn create_chat(&self, ws_id: u64, input: CreateChat) -> Result<Chat, AppError> {
        let len = input.members.len();
        let chat_type = match (&input.name, len) {
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
        };

        let chat = sqlx::query_as(
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_chat_by_id(
        &self,
        chat_id: u64,
        origin_chat: Chat,
        input: UpdateChat,
    ) -> Result<Chat, AppError> {
        let chat_type = if input.chat_type.is_some() {
            input.chat_type.unwrap()
        } else {
            origin_chat.r#type
        };

        let chat =
            sqlx::query_as("UPDATE chats SET name=$1, type=$2, members=$3 WHERE id=$4 RETURNING *")
                .bind(input.name)
                .bind(chat_type)
                .bind(input.members)
                .bind(chat_id as i64)
                .fetch_one(&self.pool)
                .await?;
        Ok(chat)
    }

    pub async fn delete_chat_by_id(&self, chat_id: u64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM chats WHERE id=$1")
            .bind(chat_id as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_chats(&self, user_id: u64, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE ws_id=$1 AND $2 = ANY(members)",
        )
        .bind(ws_id as i64)
        .bind(user_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, chat_id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE id=$1",
        )
        .bind(chat_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1 
            FROM chats 
            WHERE id=$1 AND $2 = ANY(members);
        "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(is_member.is_some())
    }

    pub async fn validate_members(&self, members: Vec<i64>) -> Result<(), AppError> {
        let len = members.len();
        let users = self.find_users_by_ids(members).await?;
        if users.len() != len {
            return Err(AppError::ChatError("Invalid members".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::fs;

    use crate::{
        config::{AuthConfig, ServerConfig},
        models::{chat::CreateChat, ChatType},
        AppConfig, AppState,
    };

    #[tokio::test]
    async fn test_create_chat() -> anyhow::Result<()> {
        let current_path = std::env::current_dir()?;
        println!("{}", current_path.display());

        let input = CreateChat {
            name: Some("test".to_string()),
            public: true,
            members: vec![1, 2],
        };

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

        // sqlx::migrate!("../migrations").run(&app_state.pool).await?;

        let chat = app_state.create_chat(1, input).await?;
        assert_eq!(chat.name, "test".to_string());
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.ws_id, 1);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&app_state.pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS chat_type;"#)
            .execute(&app_state.pool)
            .await?;

        Ok(())
    }
}
