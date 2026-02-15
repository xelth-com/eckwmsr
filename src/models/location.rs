use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::odoo_types::OdooString;

/// StockLocation mirrors Odoo 'stock.location'
/// Matches Go's `StockLocation` struct from `internal/models/stock.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_location")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub name: String,
    pub complete_name: String,
    #[sea_orm(unique)]
    pub barcode: OdooString,
    pub usage: String,
    pub location_id: Option<i64>,
    #[sea_orm(default_value = "true")]
    pub active: bool,
    #[serde(rename = "lastSyncedAt")]
    pub last_synced_at: DateTimeUtc,
    #[serde(rename = "createdAt")]
    #[sea_orm(column_type = "DateTime")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::LocationId", to = "Column::Id")]
    Parent,
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
