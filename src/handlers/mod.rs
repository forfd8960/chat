mod auth;
mod chat;
mod message;

use axum::response::IntoResponse;

#[allow(unused_imports)]
pub(crate) use auth::*;

#[allow(unused_imports)]
pub(crate) use chat::*;

#[allow(unused_imports)]
pub(crate) use message::*;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "Hello, world!"
}
