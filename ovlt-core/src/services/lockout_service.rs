use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::{entity::login_attempts, error::AppError};

const CLEANUP_WINDOW_MINUTES: i64 = 60;

pub async fn is_locked(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    email_lookup: &str,
    max_attempts: usize,
    window_minutes: i64,
) -> Result<bool, AppError> {
    let since = (Utc::now() - chrono::Duration::minutes(window_minutes)).fixed_offset();
    let attempts = login_attempts::Entity::find()
        .filter(login_attempts::Column::TenantId.eq(tenant_id))
        .filter(login_attempts::Column::EmailLookup.eq(email_lookup))
        .filter(login_attempts::Column::AttemptedAt.gte(since))
        .all(db)
        .await?;
    Ok(attempts.len() >= max_attempts)
}

pub async fn record_attempt(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    email_lookup: &str,
) -> Result<(), AppError> {
    login_attempts::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        email_lookup: Set(email_lookup.to_string()),
        attempted_at: Set(Utc::now().fixed_offset()),
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn clear_attempts(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    email_lookup: &str,
) -> Result<(), AppError> {
    login_attempts::Entity::delete_many()
        .filter(login_attempts::Column::TenantId.eq(tenant_id))
        .filter(login_attempts::Column::EmailLookup.eq(email_lookup))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn cleanup_old_attempts(db: &DatabaseConnection) -> Result<u64, AppError> {
    let cutoff = (Utc::now() - chrono::Duration::minutes(CLEANUP_WINDOW_MINUTES)).fixed_offset();
    let result = login_attempts::Entity::delete_many()
        .filter(login_attempts::Column::AttemptedAt.lt(cutoff))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
