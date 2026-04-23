use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

use crate::entity::audit_log;

/// Fire-and-forget audit log write. Never blocks the request path.
pub fn record(
    db: DatabaseConnection,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    action: impl Into<String> + Send + 'static,
    ip: Option<String>,
    metadata: Option<String>,
) {
    let action = action.into();
    tokio::spawn(async move {
        let entry = audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            action: Set(action),
            ip: Set(ip),
            metadata: Set(metadata),
            created_at: Set(Utc::now().fixed_offset()),
        };
        if let Err(e) = entry.insert(&db).await {
            tracing::warn!("audit log write failed: {e}");
        }
    });
}
