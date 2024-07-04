use axum::response::IntoResponse;

#[allow(dead_code)]
pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send message handler"
}

#[allow(dead_code)]
pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list messages handler"
}
