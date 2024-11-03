use crate::{
    error::AppError,
    models::message::{CreateMessage, ListMessages},
    AppState, User,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(msg): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state.create_message(id, user.id as u64, &msg).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}

pub(crate) async fn list_messages_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(list_msg): Json<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let msgs = state.list_messages(id, list_msg).await?;
    Ok(Json(msgs))
}
