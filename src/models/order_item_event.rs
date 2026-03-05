use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Immutable event log tracking items joining/leaving orders.
/// event_type: "added", "removed", "transferred"
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_item_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(indexed)]
    pub order_id: Uuid,
    #[sea_orm(indexed)]
    pub item_id: Uuid,
    pub event_type: String,
    pub user_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTimeUtc,
    /// For mesh sync compatibility (soft-delete pattern)
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
