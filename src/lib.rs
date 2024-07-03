mod config;

use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, post},
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
        .route("/chat", post(create_chat_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api_router)
        .with_state(state)
}

async fn index_handler() {}

async fn signin_handler() {}
async fn signup_handler() {}
async fn create_chat_handler() {}
