use crate::{error::AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::mem;

use super::Workspace;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_user_by_email(
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE email=$1")
                .bind(email)
                .fetch_optional(pool)
                .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE id=$1")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        Ok(user)
    }

    // Create new user
    pub async fn create_user(pool: &sqlx::PgPool, input: &CreateUser) -> Result<Self, AppError> {
        let user = Self::find_user_by_email(pool, &input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let ws = match Workspace::get_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, 0, pool).await?,
        };

        let pwd_hash = hash_password(&input.password)?;

        let user: User = sqlx::query_as(
            "INSERT INTO users (ws_id,fullname,email,password_hash)
            VALUES ($1,$2,$3,$4) RETURNING id,ws_id,fullname,email,created_at",
        )
        .bind(ws.id)
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(pwd_hash)
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            Workspace::update_owner(ws.id as u64, user.id as _, pool).await?;
        }

        Ok(user)
    }

    pub async fn verify_user(
        pool: &sqlx::PgPool,
        input: &SignInUser,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id,ws_id,fullname,email,password_hash,created_at FROM users WHERE email=$1",
        )
        .bind(&input.email)
        .fetch_optional(pool)
        .await?;

        if let Some(mut u) = user {
            let password_hash = mem::take(&mut u.password_hash).unwrap();
            let is_valid = verify_password(&input.password, &password_hash)?;
            if is_valid {
                Ok(Some(u))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::{CreateUser, User};
    use crate::models::Workspace;
    use anyhow::Result;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_user_should_create() -> Result<()> {
        let pool =
            PgPool::connect("postgres://db_manager:super_admin8801@localhost:5432/chat_test")
                .await?;

        sqlx::migrate!("../migrations").run(&pool).await?;

        println!("start create user...");

        let user = User::create_user(
            &pool,
            &CreateUser {
                fullname: "Bob".to_string(),
                email: "bob@acme.com".to_string(),
                workspace: "new-ws".to_string(),
                password: "test-passAbc9".to_string(),
            },
        )
        .await?;

        println!("created user: {:?}", user);
        assert_eq!(user.fullname, "Bob");
        assert_eq!(user.email, "bob@acme.com");

        let ws = Workspace::get_by_id(user.ws_id as _, &pool).await?;
        println!("created workspace: {:?}", ws);
        assert_eq!(ws.name, "new-ws");
        assert_eq!(ws.owner_id, user.id);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS chat_type;"#)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
