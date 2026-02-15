use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// ProductAlias links external codes (EAN, tracking) to internal IDs.
/// Matches Go's `ProductAlias` from `internal/models/product_alias.go`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "product_aliases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub external_code: String,
    pub internal_id: String,
    #[sea_orm(column_name = "type", column_type = "String(StringLen::None)")]
    pub r#type: String,
    #[sea_orm(default_value = "false")]
    pub is_verified: bool,
    #[sea_orm(default_value = "0")]
    pub confidence_score: i32,
    pub created_context: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
