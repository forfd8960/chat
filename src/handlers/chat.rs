use axum::response::IntoResponse;

#[allow(dead_code)]
pub(crate) async fn list_chat_handler() -> impl IntoResponse {
    "list chat handler"
}

#[allow(dead_code)]
pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create chat handler"
}

#[allow(dead_code)]
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update chat handler"
}
