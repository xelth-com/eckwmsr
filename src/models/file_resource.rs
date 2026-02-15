use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// FileResource represents a stored file (image/document) with CAS logic.
/// Matches Go's `FileResource` from `internal/models/file_resource.go`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "file_resources")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub hash: String,
    #[sea_orm(column_name = "file_name")]
    pub original_name: String,
    pub mime_type: String,
    #[sea_orm(column_name = "size")]
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_data: Option<Vec<u8>>,
    #[sea_orm(column_name = "file_path")]
    pub storage_path: String,
    #[sea_orm(column_name = "source_instance")]
    pub created_by_device: String,
    pub context: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
