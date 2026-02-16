use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mesh_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub instance_id: String,
    pub name: String,
    pub base_url: String,
    pub role: String,   // "master", "peer", "edge", "pda"
    pub status: String, // "active", "offline"
    pub last_seen: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
