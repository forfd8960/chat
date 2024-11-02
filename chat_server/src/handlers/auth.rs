use crate::{
    error::{AppError, ErrorResponse},
    models::user::{CreateUser, SignInUser},
    AppState, User,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::result::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    token: String,
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SignInUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify_user(&state.pool, &input).await?;
    match user {
        Some(u) => {
            let token = state.ek.sign(u)?;
            Ok((StatusCode::OK, Json(AuthResponse { token })).into_response())
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                msg: "invalid email or password".to_string(),
            }),
        )
            .into_response()),
    }
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create_user(&state.pool, &input).await?;
    let token = state.ek.sign(user)?;
    let resp = Json(AuthResponse { token });
    Ok((StatusCode::CREATED, resp))
}
