use crate::{error::AppError, models::Workspace, AppState, User};
use axum::{extract::State, response::IntoResponse, Extension, Json};

pub(crate) async fn list_chat_users(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.pool;
    let users = Workspace::list_all_chat_users(user.ws_id as _, pool).await?;

    Ok(Json(users))
}
