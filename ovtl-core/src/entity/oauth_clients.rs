use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "oauth_clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    #[sea_orm(column_type = "Text", unique)]
    pub client_id: String,
    #[sea_orm(column_type = "Text")]
    pub client_secret: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Json")]
    pub redirect_uris: serde_json::Value,
    #[sea_orm(column_type = "Json")]
    pub grant_types: serde_json::Value,
    #[sea_orm(column_type = "Json")]
    pub scopes: serde_json::Value,
    pub is_confidential: bool,
    pub require_consent: bool,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
