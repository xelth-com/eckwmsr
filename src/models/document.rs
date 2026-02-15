use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Document represents a structured report from a mobile device (Workflow Result)
/// Mirrors Go's `Document` from `internal/models/document.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(column_name = "document_id", primary_key, auto_increment = false)]
    #[serde(rename = "documentId")]
    pub id: Uuid,
    #[sea_orm(column_name = "type")]
    pub r#type: String, // "ManualRestock", "RMA_Result", "repair_log"
    pub status: String, // pending, processed, error
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: serde_json::Value,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,

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
