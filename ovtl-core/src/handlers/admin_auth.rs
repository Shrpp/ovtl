use axum::http::HeaderMap;
use uuid::Uuid;

use crate::{error::AppError, services::token_service};

/// Accept either X-OVTL-Admin-Key or a valid Bearer token from the master tenant.
pub fn require_admin(
    headers: &HeaderMap,
    admin_key: &Option<String>,
    jwt_secret: &str,
    master_tenant_id: Option<Uuid>,
) -> Result<(), AppError> {
    if let Some(key) = admin_key {
        let provided = headers
            .get("x-ovtl-admin-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided == key {
            return Ok(());
        }
    }

    if let Some(bearer) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        if let Ok(claims) = token_service::validate_access_token(bearer, jwt_secret) {
            if let (Some(master_id), Ok(tid)) =
                (master_tenant_id, Uuid::parse_str(&claims.tid))
            {
                if tid == master_id {
                    return Ok(());
                }
            }
        }
    }

    Err(AppError::Unauthorized)
}
