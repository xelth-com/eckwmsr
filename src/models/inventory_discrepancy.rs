use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// InventoryDiscrepancy records a mismatch between physical count and server stock.
/// Mirrors Go's `InventoryDiscrepancy` from `internal/models/inventory.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "inventory_discrepancy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub document_id: String,
    pub product_id: i64,
    pub product_barcode: String,
    pub product_name: String,
    pub product_code: String,
    pub location_id: i64,
    pub location_barcode: String,
    pub location_name: String,
    pub expected_qty: f64,
    pub counted_qty: f64,
    pub delta: f64,
    pub item_type: String,
    pub device_id: String,
    pub status: String,
    pub notes: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
