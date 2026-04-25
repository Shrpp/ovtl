use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db,
    entity::one_time_tokens,
    error::AppError,
    handlers::admin_auth,
    services::{one_time_token_service, tenant_service, user_service},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct VerificationCodeResponse {
    pub otp: String,
    pub expires_in_hours: u32,
}

fn extract_tenant_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    headers
        .get("x-ovtl-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::InvalidInput("x-ovtl-tenant-id header required".into()))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let tenant = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let users = user_service::list_all(&txn).await?;
    txn.commit().await?;

    let response: Vec<UserResponse> = users
        .into_iter()
        .map(|u| {
            let email = hefesto::decrypt(
                &u.email,
                &tenant_key,
                &state.config.master_encryption_key,
            )
            .unwrap_or_else(|_| "<encrypted>".into());
            UserResponse {
                id: u.id.to_string(),
                email,
                is_active: u.is_active,
                email_verified: u.email_verified,
                created_at: u.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    payload
        .validate()
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let tenant = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    let email_normalized = payload.email.trim().to_lowercase();
    let email_lookup = hefesto::hash_for_lookup(&email_normalized, &tenant_key)?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;

    if user_service::email_lookup_exists(&txn, &email_lookup).await? {
        txn.commit().await?;
        return Err(AppError::Conflict);
    }

    let email_encrypted = hefesto::encrypt(
        &email_normalized,
        &tenant_key,
        &state.config.master_encryption_key,
    )?;
    let password_hash = hefesto::hash_password(&payload.password)?;

    let user = user_service::create(
        &txn,
        user_service::CreateUserInput {
            tenant_id,
            email_encrypted,
            email_lookup,
            password_hash,
        },
    )
    .await?;

    txn.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id.to_string(),
            email: email_normalized,
            is_active: user.is_active,
            email_verified: user.email_verified,
            created_at: user.created_at.to_rfc3339(),
        }),
    ))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
    pub is_active: bool,
}

pub async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    payload
        .validate()
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let tenant = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;

    if let Some(email) = payload.email {
        let email_normalized = email.trim().to_lowercase();
        let email_encrypted = hefesto::encrypt(
            &email_normalized,
            &tenant_key,
            &state.config.master_encryption_key,
        )?;
        let email_lookup = hefesto::hash_for_lookup(&email_normalized, &tenant_key)?;
        user_service::update_email(&txn, id, email_encrypted, email_lookup).await?;
    }

    if let Some(password) = payload.password {
        let password_hash = hefesto::hash_password(&password)?;
        user_service::update_password(&txn, id, password_hash).await?;
    }

    user_service::set_active(&txn, id, payload.is_active).await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn deactivate_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    user_service::deactivate(&txn, id).await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_verification_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let otp = one_time_token_service::generate_otp();
    one_time_token_service::store_otp(&state.db, tenant_id, id, &otp, 48).await?;

    Ok(Json(VerificationCodeResponse {
        otp,
        expires_in_hours: 48,
    }))
}

#[derive(Debug, Serialize)]
pub struct PasswordResetTokenResponse {
    pub token: String,
    pub expires_in_minutes: i64,
}

pub async fn get_password_reset_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    admin_auth::require_admin(&headers, &state.config.admin_key, &state.config.jwt_secret, state.master_tenant_id)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let token = one_time_token_service::generate();
    let token_hash = one_time_token_service::hash(&token);
    one_time_token_service::store(
        &state.db,
        tenant_id,
        id,
        token_hash,
        one_time_tokens::TYPE_PASSWORD_RESET,
        60,
    )
    .await?;

    Ok(Json(PasswordResetTokenResponse {
        token,
        expires_in_minutes: 60,
    }))
}
