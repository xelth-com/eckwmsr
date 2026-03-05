use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A physical item/device tracked by barcodes.
/// Created automatically when a repair slot binds a barcode.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The first barcode that identified this item (typically 19-char smart code starting with 'i')
    #[sea_orm(unique)]
    pub primary_barcode: String,
    /// All known barcodes/identifiers for this item (JSON array of strings)
    #[sea_orm(column_type = "JsonBinary")]
    pub barcodes: serde_json::Value,
    pub name: Option<String>,
    /// FK to file_resources — the main/ID photo of this item
    pub main_photo_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: serde_json::Value,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
