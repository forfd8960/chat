use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub msg: String,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("upload file error: {0}")]
    UploadError(#[from] MultipartError),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("user email already registered: {0}")]
    EmailAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => StatusCode::FORBIDDEN,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::UploadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, format!("{:?}", self)).into_response()
    }
}
