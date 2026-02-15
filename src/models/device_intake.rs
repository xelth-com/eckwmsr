use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// DeviceIntake represents structured data captured during device receiving
/// Mirrors Go's `DeviceIntake` struct from `internal/models/repair_intake.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "device_intakes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,

    // Identification
    #[serde(rename = "hwb_number")]
    pub tracking_number: String,
    pub serial_number: String,

    // Condition / Checklist
    #[serde(rename = "has_psu")]
    pub has_power_supply: bool,
    pub packaging: String,
    pub cables_included: bool,

    // Raw JSON payload for flexibility
    #[sea_orm(column_type = "JsonBinary")]
    pub raw_data: serde_json::Value,

    // Integration Status
    pub odoo_repair_id: i64,
    pub sync_status: String, // pending, synced, error

    #[serde(rename = "createdAt")]
    pub created_at: DateTimeUtc,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTimeUtc,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
