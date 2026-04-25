use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "one_time_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(column_type = "Text", unique)]
    pub token_hash: String,
    #[sea_orm(column_type = "Text")]
    pub token_type: String,
    pub expires_at: DateTimeWithTimeZone,
    pub used_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub const TYPE_PASSWORD_RESET: &str = "password_reset";
pub const TYPE_EMAIL_VERIFICATION: &str = "email_verification";
