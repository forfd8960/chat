use super::{Chat, ChatType};
use crate::{error::AppError, AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub public: bool,
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
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }
}
