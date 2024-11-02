mod auth;
mod chat;
mod file;
mod message;
mod workspace;

use axum::response::IntoResponse;

#[allow(unused_imports)]
pub(crate) use auth::*;

#[allow(unused_imports)]
pub(crate) use chat::*;

#[allow(unused_imports)]
pub(crate) use message::*;

#[allow(unused_imports)]
pub(crate) use workspace::*;

#[allow(unused_imports)]
pub(crate) use file::*;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "Hello, world!"
}
