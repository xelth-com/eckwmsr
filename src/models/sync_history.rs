use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sync_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub id: Uuid,
    pub instance_id: String,
    #[sea_orm(index)]
    pub provider: String,
    #[sea_orm(index)]
    pub status: String,
    pub started_at: DateTimeUtc,
    pub completed_at: Option<DateTimeUtc>,
    #[sea_orm(default_value = "0")]
    pub duration: i32,
    #[sea_orm(default_value = "0")]
    pub created: i32,
    #[sea_orm(default_value = "0")]
    pub updated: i32,
    #[sea_orm(default_value = "0")]
    pub skipped: i32,
    #[sea_orm(default_value = "0")]
    pub errors: i32,
    #[sea_orm(column_type = "Text")]
    pub error_detail: String,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub debug_info: Option<serde_json::Value>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
