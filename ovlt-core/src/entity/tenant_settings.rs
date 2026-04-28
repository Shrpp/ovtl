use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tenant_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: Uuid,
    pub lockout_max_attempts: i32,
    pub lockout_window_minutes: i32,
    pub lockout_duration_minutes: i32,
    pub access_token_ttl_minutes: i32,
    pub refresh_token_ttl_days: i32,
    pub allow_public_registration: bool,
    pub require_email_verified: bool,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
