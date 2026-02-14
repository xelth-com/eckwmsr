use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// EntityChecksum stores computed hashes for sync comparison
/// Matches Go's `EntityChecksum` struct from `internal/models/sync.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "entity_checksums")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "entityId")]
    pub entity_id: String,
    #[serde(rename = "contentHash")]
    pub content_hash: String,
    #[serde(rename = "childrenHash")]
    pub children_hash: Option<String>,
    #[serde(rename = "fullHash")]
    pub full_hash: String,
    #[serde(rename = "childCount")]
    pub child_count: i32,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTimeUtc,
    pub source_instance: String,
    pub source_device: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTimeUtc,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
