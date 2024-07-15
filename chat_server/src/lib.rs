mod config;
mod error;
mod handlers;
mod models;
mod utils;

pub use config::AppConfig;
pub use models::User;

use anyhow::{Context, Result};
use axum::{
    routing::{get, patch, post},
    Router,
};
use error::AppError;

use handlers::{
    create_chat_handler, delete_chat_handler, index_handler, list_chat_handler,
    list_messages_handler, send_message_handler, signin_handler, signup_handler,
    update_chat_handler,
};

use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};
use utils::{DecodingKey, EncodingKey};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>, // make AppState clone to be Lightweight
}

#[allow(dead_code)]
pub(crate) struct AppStateInner {
    pub(crate) pool: PgPool,
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.private_key).context("load DecodingKey failed")?;
        let ek = EncodingKey::load(&config.auth.public_key).context("load EncodingKey failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect DB failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                pool,
                config,
                dk,
                ek,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

pub async fn get_router(conf: AppConfig) -> Result<axum::Router, AppError> {
    let state = AppState::try_new(conf).await?;

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

    Ok(Router::new()
        .route("/", get(index_handler))
        .nest("/api", api_router)
        .with_state(state))
}
