use std::ops::Deref;

use anyhow::Result;
use jwt_simple::prelude::*;

use crate::{error::AppError, User};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

#[allow(dead_code)]
impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(user: User, key: &EncodingKey) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);

        Ok(key.sign(claims)?)
    }
}

#[allow(dead_code)]
impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(token: &str, key: &DecodingKey) -> Result<User, AppError> {
        let options = jwt_simple::common::VerificationOptions {
            allowed_issuers: Some(HashSet::from([JWT_ISS.to_string()])),
            allowed_audiences: Some(HashSet::from([JWT_AUD.to_string()])),
            ..Default::default()
        };

        let claims = key.verify_token::<User>(token, Some(options))?;
        Ok(claims.custom)
    }
}

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
