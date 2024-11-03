use std::path::PathBuf;

use axum::{
    extract::{Multipart, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tracing::info;

use crate::{error::AppError, models::file::ChatFile, AppState, User};

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = PathBuf::from(&state.config.server.base_dir).join(ws_id.to_string());

    let mut files = vec![];

    while let Some(field) = multipart.next_field().await? {
        let f_name = field.file_name().map(|name| name.to_string());
        if let (Some(file_name), Ok(data)) = (f_name, field.bytes().await) {
            let chat_file = ChatFile::new(&file_name, &data);

            info!("Uploaded file: {}, size: {} bytes", file_name, data.len());
            let path = chat_file.path(&base_dir);
            if path.exists() {
                info!("{}, path: {:?} is already exists", file_name, path);
            } else {
                fs::create_dir_all(path.parent().expect("xxx")).await?;
                fs::write(path, data).await?;
            }
            files.push(chat_file.url());
        }
    }

    Ok(Json(files))
}

pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::Unauthorized);
    }

    let base_dir = PathBuf::from(&state.config.server.base_dir).join(ws_id.to_string());
    let file_path = base_dir.join(path);
    if !file_path.exists() {
        return Err(AppError::NotFound("file".to_string()));
    }

    let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
    let mut header = HeaderMap::new();
    header.insert("Content-Type", mime.to_string().parse().unwrap());

    let body = fs::read(file_path).await?;
    Ok((header, body))
}
