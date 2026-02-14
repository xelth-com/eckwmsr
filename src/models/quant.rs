use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// StockQuant represents inventory levels per product/location
/// Matches Go's `StockQuant` struct from `internal/models/stock.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_quant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[serde(rename = "productId")]
    pub product_id: i64,
    #[serde(rename = "locationId")]
    pub location_id: i64,
    #[serde(rename = "lotId")]
    pub lot_id: Option<i64>,
    #[serde(rename = "packageId")]
    pub package_id: Option<i64>,
    pub quantity: f64,
    pub reserved_quantity: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
