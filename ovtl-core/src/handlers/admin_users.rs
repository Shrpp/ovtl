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
    error::AppError,
    handlers::admin_auth,
    services::{tenant_service, user_service},
    state::AppState,
};

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
            created_at: user.created_at.to_rfc3339(),
        }),
    ))
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
