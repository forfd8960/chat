use crate::{error::AppError, AppState, User};
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

impl AppState {
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE email=$1")
                .bind(email)
                .fetch_optional(&self.pool)
                .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE id=$1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(user)
    }

    pub async fn find_users_by_ids(&self, ids: Vec<i64>) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as(
            "SELECT id,ws_id,fullname,email,created_at FROM users WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    // Create new user
    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let ws = match self.get_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
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
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            self.update_wokrspce_owner(ws.id as u64, user.id as _)
                .await?;
        }

        Ok(user)
    }

    pub async fn verify_user(&self, input: &SignInUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id,ws_id,fullname,email,password_hash,created_at FROM users WHERE email=$1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
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
    use super::CreateUser;
    use crate::{
        config::{AuthConfig, ServerConfig},
        AppConfig, AppState,
    };
    use anyhow::Result;
    use tokio::fs;

    #[tokio::test]
    async fn test_user_should_create() -> Result<()> {
        let p_key = fs::read_to_string("./fixture/private.pem").await?;
        let d_key = fs::read_to_string("./fixture/public.pem").await?;

        let config = AppConfig {
            server: ServerConfig {
                port: 8088,
                db_url: "postgres://db_manager:super_admin8801@localhost:5432/chat_test"
                    .to_string(),
                base_dir: "/tmp/chat".to_string(),
            },
            auth: AuthConfig {
                private_key: p_key,
                public_key: d_key,
            },
        };

        let app_state = AppState::try_new(config).await?;
        let pool = app_state.pool.clone();

        println!("start create user...");
        let user = app_state
            .create_user(&CreateUser {
                fullname: "Bob".to_string(),
                email: "bob@acme.com".to_string(),
                workspace: "new-ws".to_string(),
                password: "test-passAbc9".to_string(),
            })
            .await?;

        println!("created user: {:?}", user);
        assert_eq!(user.fullname, "Bob");
        assert_eq!(user.email, "bob@acme.com");

        let ws = app_state.get_workspace_by_id(user.ws_id as _).await?;
        println!("created workspace: {:?}", ws);
        assert_eq!(ws.name, "new-ws");
        assert_eq!(ws.owner_id, user.id);

        sqlx::query(r#"TRUNCATE TABLE users, workspaces, chats, messages;"#)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
