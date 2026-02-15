use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Order represents a unified order/request table for RMA and repairs
/// Mirrors Go's `Order` struct from `internal/models/order.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "orders")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub order_number: String,

    // Order classification
    #[sea_orm(indexed)]
    pub order_type: String, // "rma" | "repair"

    // Customer information (for RMA)
    #[sea_orm(indexed)]
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,

    // Item information
    pub item_id: Option<i32>,
    #[sea_orm(indexed)]
    pub product_sku: String,
    pub product_name: String,
    #[sea_orm(indexed)]
    pub serial_number: String,
    pub purchase_date: Option<DateTimeUtc>,

    // Problem/Issue description
    #[sea_orm(column_type = "Text")]
    pub issue_description: String,
    #[sea_orm(column_type = "Text")]
    pub diagnosis_notes: String,

    // Assignment (Go uses *uint pointing to UserAuth; we use Option<String> for flexibility)
    pub assigned_to: Option<String>,

    // Status and priority
    pub status: String,
    pub priority: String,

    // Repair-specific fields
    #[sea_orm(column_type = "Text")]
    pub repair_notes: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub parts_used: serde_json::Value,
    pub labor_hours: f64,
    pub total_cost: f64,

    // Resolution
    #[sea_orm(column_type = "Text")]
    pub resolution: String,
    #[sea_orm(column_type = "Text")]
    pub notes: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: serde_json::Value,

    // RMA-specific fields
    pub rma_reason: String,
    pub is_refund_requested: bool,

    // Timestamps
    pub started_at: Option<DateTimeUtc>,
    pub completed_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
