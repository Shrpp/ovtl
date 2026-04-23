use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{error::AppError, services::tenant_service, state::AppState};

const TENANT_HEADER: &str = "x-ovtl-tenant-id";

#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    /// Decrypted per-tenant encryption key (lives only in memory).
    pub tenant_key: String,
}

pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let tenant_id = req
        .headers()
        .get(TENANT_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Unauthorized)?;

    let record = tenant_service::find_active(&state.db, tenant_id).await?;

    // Tenant key is double-envelope encrypted: inner key = TENANT_WRAP_KEY, outer = MASTER_ENCRYPTION_KEY.
    let tenant_key = hefesto::decrypt(
        &record.encryption_key_encrypted,
        &state.config.tenant_wrap_key,
        &state.config.master_encryption_key,
    )?;

    req.extensions_mut().insert(TenantContext {
        tenant_id,
        tenant_key,
    });

    Ok(next.run(req).await)
}
