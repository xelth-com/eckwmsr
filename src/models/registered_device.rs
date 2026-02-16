use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "registered_devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(rename = "deviceId")]
    pub device_id: String,

    pub name: Option<String>,

    #[sea_orm(column_name = "public_key")]
    #[serde(rename = "publicKey")]
    pub public_key: Option<String>,

    pub status: Option<String>, // "pending", "active", "blocked"

    #[sea_orm(column_name = "home_instance_id")]
    #[serde(rename = "homeInstanceId")]
    pub home_instance_id: Option<String>,

    #[sea_orm(column_name = "last_seen_at")]
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: Option<DateTimeUtc>,

    #[sea_orm(column_name = "created_at")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTimeUtc>,

    #[sea_orm(column_name = "updated_at")]
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTimeUtc>,

    #[sea_orm(column_name = "deleted_at")]
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
