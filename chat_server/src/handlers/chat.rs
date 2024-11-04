use crate::{
    error::AppError,
    models::{
        chat::{CreateChat, UpdateChat},
        ChatType,
    },
    AppState, User,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.list_chats(user.id as u64, user.ws_id as u64).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(user.ws_id as u64, input).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat: {} not found", id))),
    }
}

pub(crate) async fn update_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    if chat.is_none() {
        return Err(AppError::NotFound("chat: {id} not found".to_string()));
    }

    let origin_chat = chat.unwrap();
    let update_chat = input.clone();

    let input_type = input.chat_type;
    if input_type.is_some() {
        let update_type = input_type.unwrap();
        if origin_chat.r#type == ChatType::PrivateChannel && update_type == ChatType::PublicChannel
        {
            return Err(AppError::ChatError(
                "conn't convert private to public".to_string(),
            ));
        }

        if origin_chat.r#type == ChatType::Group && update_type == ChatType::Single {
            return Err(AppError::ChatError(
                "conn't convert group to single".to_string(),
            ));
        }
    }

    state.validate_members(input.members.clone()).await?;

    let chat = state
        .update_chat_by_id(id, origin_chat, update_chat)
        .await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    if chat.is_none() {
        return Err(AppError::NotFound("chat: {id} not found".to_string()));
    }

    let chat = chat.unwrap();
    if chat.ws_id != user.ws_id {
        return Err(AppError::ChatError(
            "chat does not belong to you".to_string(),
        ));
    }

    state.delete_chat_by_id(id).await?;
    Ok(())
}
