use crate::{error::AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::mem;

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

        let pwd_hash = hash_password(&input.password)?;

        let user = sqlx::query_as(
            "INSERT INTO users (fullname,email,password_hash)
            VALUES ($1,$2,$3) RETURNING id,ws_id,fullname,email,created_at",
        )
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(pwd_hash)
        .fetch_one(pool)
        .await?;

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
