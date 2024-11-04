use crate::{error::AppError, AppState, User};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};

pub(crate) async fn list_chat_users(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.list_all_chat_users(user.ws_id as _).await?;
    Ok(Json(users))
}

pub(crate) async fn get_workspace_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(ws_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::Unauthorized);
    }

    let workspace = state.get_workspace_by_id(ws_id as _).await?;
    Ok(Json(workspace))
}
