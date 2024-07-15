use crate::{
    error::AppError,
    models::{CreateUser, SignInUser},
    AppState, User,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::result::Result;

#[allow(dead_code)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create_user(&state.pool, &input).await?;
    let token = &state.ek.sign(user)?;

    Ok((StatusCode::CREATED, token.to_owned()))
}

#[allow(dead_code)]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<SignInUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify_user(&state.pool, &input).await?;
    match user {
        Some(u) => {
            let token = &state.ek.sign(u)?;
            Ok((StatusCode::OK, token.to_owned()))
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            "invalid email or password".to_string(),
        )),
    }
}
