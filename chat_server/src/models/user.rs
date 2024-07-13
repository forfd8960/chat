use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::{error::AppError, User};

impl User {
    pub async fn find_by_email(pool: &sqlx::PgPool, email: &str) -> Result<Option<Self>, AppError> {
        let r: Option<User> =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE email=$1")
                .bind(email)
                .fetch_optional(pool)
                .await?;

        Ok(r)
    }

    // Create new user
    pub async fn create_user(
        pool: &sqlx::PgPool,
        email: &str,
        fullname: &str,
        password: &str,
    ) -> Result<Self, AppError> {
        let pwd = hash_password(password)?;

        let r: User = sqlx::query_as(
            "INSERT INTO users (fullname,email,password) VALUES ($1,$2,$3) RETURNING id,fullname,email,created_at",
        )
        .bind(fullname)
        .bind(email)
        .bind(pwd)
        .fetch_one(pool)
        .await?;

        Ok(r)
    }

    pub async fn verify_user(
        pool: &sqlx::PgPool,
        email: &str,
        password: &str,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id,ws_id,fullname,email,password_hash,created_at FROM users WHERE email=$1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        if let Some(mut u) = user {
            let password_hash = mem::take(&mut u.password_hash).unwrap();
            let is_valid = verify_password(password, &password_hash)?;
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
