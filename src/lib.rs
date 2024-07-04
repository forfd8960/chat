mod config;
mod handlers;

use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, patch, post},
    Router,
};
pub use config::AppConfig;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

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

async fn index_handler() {}

async fn signin_handler() {}
async fn signup_handler() {}
async fn create_chat_handler() {}
async fn list_chat_handler() {}
async fn update_chat_handler() {}
async fn delete_chat_handler() {}
async fn send_message_handler() {}
async fn list_messages_handler() {}
