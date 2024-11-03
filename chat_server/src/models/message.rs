use std::path::PathBuf;

use super::Message;
use crate::{error::AppError, AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
    pub async fn create_message(
        &self,
        chat_id: u64,
        send_id: u64,
        msg: &CreateMessage,
    ) -> Result<Message, AppError> {
        if msg.content.is_empty() {
            return Err(AppError::MessageError("content is empty".to_string()));
        }

        let mut files = vec![];

        let base_dir = self.config.server.base_dir.clone();

        for file in &msg.files {
            let file_path = format!("{}/{}", base_dir, file);
            let p = PathBuf::from(&file_path);
            if !p.exists() {
                return Err(AppError::MessageError("file is not exists".to_string()));
            }

            files.push(file_path);
        }

        let message = sqlx::query_as(
            "INSERT INTO messages (chat_id, sender_id, content, files) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(chat_id as i64)
        .bind(send_id as i64)
        .bind(msg.content.clone())
        .bind(files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn list_messages(
        &self,
        chat_id: u64,
        input: ListMessages,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);
        let limit = match input.limit {
            0 => i64::MAX,
            1..=100 => input.limit as i64,
            _ => 100,
        };

        let messages = sqlx::query_as(
            "SELECT id, chat_id, sender_id, content, files, created_at FROM messages WHERE chat_id=$1 AND id < $2 ORDER BY id DESC LIMIT $3",
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(limit)
        .fetch_all(&self.pool).await?;

        Ok(messages)
    }
}
