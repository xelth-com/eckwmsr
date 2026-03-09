use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// StockQuant represents inventory levels per product/location
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_quant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[serde(rename = "productId")]
    pub product_id: Uuid,
    #[serde(rename = "locationId")]
    pub location_id: Uuid,
    #[serde(rename = "lotId")]
    pub lot_id: Option<Uuid>,
    #[serde(rename = "packageId")]
    pub package_id: Option<Uuid>,
    pub quantity: f64,
    #[sea_orm(column_name = "reserved_qty")]
    pub reserved_quantity: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
