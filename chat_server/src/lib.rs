mod config;
mod error;
mod handlers;
mod models;
mod utils;

use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, patch, post},
    Router,
};
pub use config::AppConfig;
use handlers::{
    create_chat_handler, delete_chat_handler, index_handler, list_chat_handler,
    list_messages_handler, send_message_handler, signin_handler, signup_handler,
    update_chat_handler,
};

pub use models::User;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>, // make AppState clone to be Lightweight
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

pub fn get_router(conf: AppConfig) -> axum::Router {
    let state = AppState::new(conf);

    let api_router = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", post(create_chat_handler).get(list_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_messages_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api_router)
        .with_state(state)
}
