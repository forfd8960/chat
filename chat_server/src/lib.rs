mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

pub use config::AppConfig;
use middlewares::{auth::verify_token, set_layer};
pub use models::User;

use anyhow::{Context, Result};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};
use error::AppError;

use handlers::{
    create_chat_handler, delete_chat_handler, file_handler, get_chat_handler,
    get_workspace_handler, index_handler, list_chat_handler, list_chat_users,
    list_messages_handler, send_message_handler, signin_handler, signup_handler,
    update_chat_handler, upload_handler,
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
        tokio::fs::create_dir_all(&config.server.base_dir).await?;

        let ek = EncodingKey::load(&config.auth.private_key).context("load EncodingKey failed")?;
        let dk = DecodingKey::load(&config.auth.public_key).context("load DecodingKey failed")?;

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

pub trait TokenVeirfy {
    type Error: fmt::Debug;
    fn vetify(&self, token: &str) -> Result<User, Self::Error>;
}

impl TokenVeirfy for AppState {
    type Error = AppError;
    fn vetify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

pub async fn get_router(conf: AppConfig) -> Result<axum::Router, AppError> {
    let state = AppState::try_new(conf).await?;

    let api_router = Router::new()
        .route("/users", get(list_chat_users))
        .route("/chats", post(create_chat_handler).get(list_chat_handler))
        .route("/uploadfile", post(upload_handler))
        .route("/download/:ws_id/*path", get(file_handler))
        .route(
            "/chats/:id",
            get(get_chat_handler)
                .put(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_messages_handler))
        .route("/workspaces/:ws_id", get(get_workspace_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api_router)
        .with_state(state);

    Ok(set_layer(app))
}
