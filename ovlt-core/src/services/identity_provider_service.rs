use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::{entity::identity_providers, error::AppError, services::client_service};

pub struct CreateIdpInput {
    pub tenant_id: Uuid,
    pub provider: String,
    pub client_id: String,
    pub client_secret_enc: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

pub async fn create(
    txn: &DatabaseTransaction,
    input: CreateIdpInput,
) -> Result<identity_providers::Model, AppError> {
    Ok(identity_providers::ActiveModel {
        tenant_id: Set(input.tenant_id),
        provider: Set(input.provider),
        client_id: Set(input.client_id),
        client_secret_enc: Set(input.client_secret_enc),
        redirect_url: Set(input.redirect_url),
        scopes: Set(serde_json::json!(input.scopes)),
        enabled: Set(true),
        created_at: Set(Utc::now().fixed_offset()),
        ..Default::default()
    }
    .insert(txn)
    .await?)
}

pub async fn find_by_provider(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    provider: &str,
) -> Result<Option<identity_providers::Model>, AppError> {
    Ok(identity_providers::Entity::find()
        .filter(identity_providers::Column::TenantId.eq(tenant_id))
        .filter(identity_providers::Column::Provider.eq(provider))
        .filter(identity_providers::Column::Enabled.eq(true))
        .one(txn)
        .await?)
}

pub async fn list(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
) -> Result<Vec<identity_providers::Model>, AppError> {
    Ok(identity_providers::Entity::find()
        .filter(identity_providers::Column::TenantId.eq(tenant_id))
        .all(txn)
        .await?)
}

pub async fn update(
    txn: &DatabaseTransaction,
    id: Uuid,
    client_id: String,
    client_secret_enc: String,
    redirect_url: String,
    scopes: Vec<String>,
    enabled: bool,
) -> Result<identity_providers::Model, AppError> {
    let record = identity_providers::Entity::find_by_id(id)
        .one(txn)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut active = record.into_active_model();
    active.client_id = Set(client_id);
    active.client_secret_enc = Set(client_secret_enc);
    active.redirect_url = Set(redirect_url);
    active.scopes = Set(serde_json::json!(scopes));
    active.enabled = Set(enabled);
    Ok(active.update(txn).await?)
}

pub async fn delete(txn: &DatabaseTransaction, id: Uuid) -> Result<(), AppError> {
    identity_providers::Entity::delete_by_id(id)
        .exec(txn)
        .await?;
    Ok(())
}

pub fn scopes_from_value(val: &serde_json::Value) -> Vec<String> {
    client_service::scopes_to_vec(val)
}
