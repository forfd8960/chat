use crate::{error::AppError, models::file::ChatFile, AppState, User};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Extension, Json,
};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

#[allow(dead_code)]
pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send message handler"
}

#[allow(dead_code)]
pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list messages handler"
}
