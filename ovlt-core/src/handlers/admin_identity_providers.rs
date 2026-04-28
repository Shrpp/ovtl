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
    services::{identity_provider_service, tenant_service},
    state::AppState,
};

fn extract_tenant_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    headers
        .get("x-ovlt-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::InvalidInput("x-ovlt-tenant-id header required".into()))
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    admin_auth::require_admin(
        headers,
        &state.config.admin_key,
        &state.config.jwt_secret,
        state.master_tenant_id,
    )
    .map(|_| ())
}

#[derive(Debug, Serialize)]
pub struct IdpResponse {
    pub id: String,
    pub provider: String,
    pub client_id: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateIdpRequest {
    #[validate(length(min = 1, max = 32))]
    pub provider: String,
    #[validate(length(min = 1))]
    pub client_id: String,
    #[validate(length(min = 1))]
    pub client_secret: String,
    #[validate(url)]
    pub redirect_url: String,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateIdpRequest {
    #[validate(length(min = 1))]
    pub client_id: String,
    #[validate(length(min = 1))]
    pub client_secret: String,
    #[validate(url)]
    pub redirect_url: String,
    pub scopes: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

pub async fn list_idps(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let idps = identity_provider_service::list(&txn, tenant_id).await?;
    txn.commit().await?;

    let resp: Vec<IdpResponse> = idps
        .into_iter()
        .map(|idp| IdpResponse {
            id: idp.id.to_string(),
            provider: idp.provider,
            client_id: idp.client_id,
            redirect_url: idp.redirect_url,
            scopes: identity_provider_service::scopes_from_value(&idp.scopes),
            enabled: idp.enabled,
            created_at: idp.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(resp))
}

pub async fn create_idp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateIdpRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    payload.validate().map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let tenant_record = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant_record.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;
    let secret_enc = hefesto::encrypt(
        &payload.client_secret,
        &tenant_key,
        &state.config.master_encryption_key,
    )?;

    let scopes = payload
        .scopes
        .unwrap_or_else(|| vec!["openid".into(), "email".into(), "profile".into()]);

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let idp = identity_provider_service::create(
        &txn,
        identity_provider_service::CreateIdpInput {
            tenant_id,
            provider: payload.provider,
            client_id: payload.client_id,
            client_secret_enc: secret_enc,
            redirect_url: payload.redirect_url,
            scopes,
        },
    )
    .await?;
    txn.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(IdpResponse {
            id: idp.id.to_string(),
            provider: idp.provider,
            client_id: idp.client_id,
            redirect_url: idp.redirect_url,
            scopes: identity_provider_service::scopes_from_value(&idp.scopes),
            enabled: idp.enabled,
            created_at: idp.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn update_idp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIdpRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    payload.validate().map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let tenant_record = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant_record.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;
    let secret_enc = hefesto::encrypt(
        &payload.client_secret,
        &tenant_key,
        &state.config.master_encryption_key,
    )?;

    let scopes = payload
        .scopes
        .unwrap_or_else(|| vec!["openid".into(), "email".into(), "profile".into()]);

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let idp = identity_provider_service::update(
        &txn,
        id,
        payload.client_id,
        secret_enc,
        payload.redirect_url,
        scopes,
        payload.enabled.unwrap_or(true),
    )
    .await?;
    txn.commit().await?;

    Ok(Json(IdpResponse {
        id: idp.id.to_string(),
        provider: idp.provider,
        client_id: idp.client_id,
        redirect_url: idp.redirect_url,
        scopes: identity_provider_service::scopes_from_value(&idp.scopes),
        enabled: idp.enabled,
        created_at: idp.created_at.to_rfc3339(),
    }))
}

pub async fn delete_idp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, &headers)?;
    let tenant_id = extract_tenant_id(&headers)?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    identity_provider_service::delete(&txn, id).await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
