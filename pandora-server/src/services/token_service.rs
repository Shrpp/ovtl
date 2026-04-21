use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{entity::refresh_tokens, error::AppError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub tid: String,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn generate_access_token(
    user_id: Uuid,
    tenant_id: Uuid,
    email: &str,
    secret: &str,
    expiration_minutes: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        tid: tenant_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + expiration_minutes * 60,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::TokenError(e.to_string()))
}

pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::TokenError(e.to_string()))
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn store_refresh_token(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expiration_days: i64,
) -> Result<(), AppError> {
    let expires_at = (Utc::now() + chrono::Duration::days(expiration_days)).fixed_offset();
    let model = refresh_tokens::ActiveModel {
        tenant_id: Set(tenant_id),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at),
        ..Default::default()
    };
    model.insert(txn).await?;
    Ok(())
}
