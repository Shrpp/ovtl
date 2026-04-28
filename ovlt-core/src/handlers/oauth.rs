use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use serde::Deserialize;

use crate::{
    db,
    error::AppError,
    handlers::login::TokenResponse,
    middleware::tenant::TenantContext,
    services::{identity_provider_service, oauth_service, tenant_service, token_service},
    state::AppState,
};

/// `GET /auth/:provider` — redirects to provider's consent screen.
/// Looks up IdP config from DB (per-tenant), falls back to global config env vars.
pub async fn authorize(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let creds = resolve_idp_creds(&state, ctx.tenant_id, &provider).await?;
    let (auth_url, _) = oauth_service::build_authorize_url(
        &provider,
        &creds,
        ctx.tenant_id,
        &state.config.jwt_secret,
        None,
    )?;
    Ok(Redirect::to(&auth_url))
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// `GET /auth/:provider/callback` — no tenant header; tenant_id extracted from state param.
pub async fn callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = oauth_service::verify_state(&params.state, &state.config.jwt_secret)
        .ok_or(AppError::Unauthorized)?;

    let record = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &record.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    let creds = resolve_idp_creds(&state, tenant_id, &provider).await?;
    let provider_token = oauth_service::exchange_code(&provider, &params.code, &creds).await?;
    let profile = oauth_service::fetch_profile(&provider, &provider_token).await?;

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let user = oauth_service::find_or_create_user(
        &txn,
        tenant_id,
        &tenant_key,
        &state.config.master_encryption_key,
        &provider,
        &profile,
    )
    .await?;

    let email = hefesto::decrypt(&user.email, &tenant_key, &state.config.master_encryption_key)?;
    let access_token = token_service::generate_access_token(
        user.id,
        tenant_id,
        &email,
        vec![],
        vec![],
        std::collections::HashMap::new(),
        &state.config.jwt_secret,
        state.config.jwt_expiration_minutes,
    )?;
    let refresh_token = token_service::generate_refresh_token();
    let token_hash = token_service::hash_refresh_token(&refresh_token);
    token_service::store_refresh_token(
        &txn,
        tenant_id,
        user.id,
        token_hash,
        state.config.refresh_token_expiration_days,
    )
    .await?;

    txn.commit().await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        expires_in: state.config.jwt_expiration_minutes * 60,
    }))
}

/// Resolves IdP credentials: DB config first, global env vars as fallback.
async fn resolve_idp_creds(
    state: &AppState,
    tenant_id: uuid::Uuid,
    provider: &str,
) -> Result<oauth_service::IdpCredentials, AppError> {
    // Try DB-stored IdP config (no RLS bypass needed — we set tenant context)
    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;
    let db_idp = identity_provider_service::find_by_provider(&txn, tenant_id, provider).await?;
    txn.commit().await?;

    if let Some(idp) = db_idp {
        let tenant_record = tenant_service::find_active(&state.db, tenant_id).await?;
        let tenant_key = hefesto::decrypt(
            &tenant_record.encryption_key_encrypted,
            &state.config.tenant_wrap_key,
            &state.config.master_encryption_key,
        )?;
        let client_secret = hefesto::decrypt(
            &idp.client_secret_enc,
            &tenant_key,
            &state.config.master_encryption_key,
        )?;
        return Ok(oauth_service::IdpCredentials {
            client_id: idp.client_id,
            client_secret,
            redirect_url: idp.redirect_url,
        });
    }

    // Fallback: global env var config
    let cfg = state
        .config
        .oauth_for(provider)
        .ok_or_else(|| AppError::InvalidInput(format!("{provider} not configured for this tenant")))?;

    Ok(oauth_service::IdpCredentials {
        client_id: cfg.client_id.clone(),
        client_secret: cfg.client_secret.clone(),
        redirect_url: cfg.redirect_url.clone(),
    })
}
