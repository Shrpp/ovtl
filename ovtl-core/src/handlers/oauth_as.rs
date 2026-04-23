use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db,
    entity::authorization_codes,
    error::AppError,
    services::{client_service, tenant_service, token_service, user_service},
    state::AppState,
};

// ── /oauth/authorize ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
}

pub async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AuthorizeParams>,
) -> Result<impl IntoResponse, AppError> {
    // Extract and validate Bearer token — API-first: user must be pre-authenticated.
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = token_service::validate_access_token(bearer, &state.config.jwt_secret)?;
    let tenant_id = Uuid::parse_str(&claims.tid)
        .map_err(|_| AppError::TokenError("invalid tenant_id in token".into()))?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("invalid sub in token".into()))?;

    if params.response_type != "code" {
        return Err(AppError::InvalidInput("response_type must be code".into()));
    }
    if params.code_challenge_method != "S256" {
        return Err(AppError::InvalidInput(
            "code_challenge_method must be S256".into(),
        ));
    }
    if params.code_challenge.is_empty() {
        return Err(AppError::InvalidInput("code_challenge required".into()));
    }

    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;

    let client = client_service::find_by_client_id(&txn, &params.client_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if !client.is_active {
        txn.commit().await?;
        return Err(AppError::Unauthorized);
    }

    if !client_service::redirect_uri_allowed(&client, &params.redirect_uri) {
        txn.commit().await?;
        return Err(AppError::InvalidInput("redirect_uri not allowed".into()));
    }

    let code = hex::encode(Sha256::digest(Uuid::new_v4().to_string().as_bytes()))
        + &hex::encode(Sha256::digest(Uuid::new_v4().to_string().as_bytes()));

    let scopes = params
        .scope
        .as_deref()
        .unwrap_or("openid")
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    let expires_at = (Utc::now() + chrono::Duration::minutes(10)).fixed_offset();

    authorization_codes::ActiveModel {
        code: Set(code.clone()),
        tenant_id: Set(tenant_id),
        client_id: Set(params.client_id),
        user_id: Set(user_id),
        redirect_uri: Set(params.redirect_uri.clone()),
        scopes: Set(serde_json::json!(scopes)),
        code_challenge: Set(params.code_challenge),
        nonce: Set(params.nonce),
        expires_at: Set(expires_at),
        used_at: Set(None),
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(s) = &params.state {
        redirect_url.push_str(&format!("&state={s}"));
    }

    Ok(Redirect::to(&redirect_url))
}

// ── /oauth/token ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub scope: String,
}

#[derive(Serialize)]
struct IdTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    iat: i64,
    exp: i64,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
}

pub async fn token(
    State(state): State<AppState>,
    Form(req): Form<TokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.grant_type != "authorization_code" {
        return Err(AppError::InvalidInput("unsupported grant_type".into()));
    }

    // 1. Fetch the authorization code (no RLS — codes table has no policy).
    let auth_code = authorization_codes::Entity::find_by_id(&req.code)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::InvalidInput("invalid code".into()))?;

    let now = Utc::now().fixed_offset();
    if auth_code.expires_at < now {
        return Err(AppError::InvalidInput("code expired".into()));
    }
    if auth_code.used_at.is_some() {
        return Err(AppError::InvalidInput("code already used".into()));
    }
    if auth_code.redirect_uri != req.redirect_uri {
        return Err(AppError::InvalidInput("redirect_uri mismatch".into()));
    }

    // 2. Verify PKCE S256: BASE64URL(SHA256(code_verifier)) == stored code_challenge.
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(req.code_verifier.as_bytes()));
    if computed != auth_code.code_challenge {
        return Err(AppError::InvalidInput("invalid code_verifier".into()));
    }

    let tenant_id = auth_code.tenant_id;
    let txn = db::begin_tenant_txn(&state.db, tenant_id).await?;

    // 3. Validate client (RLS scopes this to the right tenant).
    let client = client_service::find_by_client_id(&txn, &req.client_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !client.is_active {
        txn.commit().await?;
        return Err(AppError::Unauthorized);
    }
    if client.client_id != auth_code.client_id {
        txn.commit().await?;
        return Err(AppError::Unauthorized);
    }

    if client.is_confidential {
        let secret = req
            .client_secret
            .as_deref()
            .ok_or(AppError::Unauthorized)?;
        if !client_service::verify_secret(secret, &client.client_secret) {
            txn.commit().await?;
            return Err(AppError::Unauthorized);
        }
    }

    // 4. Mark code used.
    let mut active: authorization_codes::ActiveModel = auth_code.clone().into();
    active.used_at = Set(Some(now));
    active.update(&txn).await?;

    // 5. Get tenant key to decrypt user email for id_token.
    let tenant_record = tenant_service::find_active(&state.db, tenant_id).await?;
    let tenant_key = hefesto::decrypt(
        &tenant_record.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    let user = user_service::find_by_id(&txn, auth_code.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let email_plain = hefesto::decrypt(
        &user.email,
        &tenant_key,
        &state.config.master_encryption_key,
    )?;

    // 6. Issue access + refresh tokens.
    let access_token = token_service::generate_access_token(
        user.id,
        tenant_id,
        &email_plain,
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

    // 7. Sign id_token (RS256).
    let scope_str = client_service::scopes_to_vec(&auth_code.scopes).join(" ");
    let id_claims = IdTokenClaims {
        iss: state.config.ovtl_issuer.clone(),
        sub: user.id.to_string(),
        aud: req.client_id.clone(),
        iat: Utc::now().timestamp(),
        exp: Utc::now().timestamp() + 3600,
        email: email_plain,
        nonce: auth_code.nonce,
    };

    let id_token = state
        .jwk
        .sign_id_token(&id_claims)
        .map_err(|e| AppError::TokenError(e.to_string()))?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        id_token,
        token_type: "Bearer",
        expires_in: state.config.jwt_expiration_minutes * 60,
        scope: scope_str,
    }))
}

// ── /oauth/introspect ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
}

pub async fn introspect(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(req): Form<IntrospectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let Some(expected_key) = &state.config.admin_key else {
        return Err(AppError::NotFound);
    };
    let provided = headers
        .get("x-ovtl-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided != expected_key {
        return Err(AppError::Unauthorized);
    }

    match token_service::validate_access_token(&req.token, &state.config.jwt_secret) {
        Ok(claims) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "active": true,
                "sub": claims.sub,
                "tid": claims.tid,
                "email": claims.email,
                "jti": claims.jti,
                "exp": claims.exp,
                "iat": claims.iat,
            })),
        )),
        Err(_) => Ok((StatusCode::OK, Json(serde_json::json!({ "active": false })))),
    }
}
