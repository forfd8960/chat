mod user;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,

    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}