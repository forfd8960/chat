use std::path::PathBuf;

use super::Message;
use crate::{error::AppError, AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
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
}
