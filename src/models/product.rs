use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::odoo_types::OdooString;

/// ProductProduct mirrors Odoo 'product.product'
/// Matches Go's `ProductProduct` struct from `internal/models/product.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "product_product")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub default_code: OdooString,
    pub barcode: OdooString,
    pub name: String,
    #[sea_orm(default_value = "true")]
    pub active: bool,
    #[sea_orm(column_name = "type", column_type = "String(StringLen::None)")]
    pub r#type: String,
    pub list_price: f64,
    pub standard_price: f64,
    pub weight: f64,
    pub volume: f64,
    pub write_date: DateTimeUtc,
    #[serde(rename = "lastSyncedAt")]
    pub last_synced_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
