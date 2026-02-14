use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// WarehouseRack represents a storage rack on the visual warehouse blueprint.
/// Matches Go's `WarehouseRack` struct from `internal/models/rack.go`.
/// Critical for TSP route optimization (pos_x, pos_y coordinates).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "warehouse_racks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub prefix: Option<String>,
    #[sea_orm(default_value = "1")]
    pub columns: i32,
    #[sea_orm(default_value = "1")]
    pub rows: i32,
    pub start_index: i32,
    #[sea_orm(default_value = "0")]
    pub sort_order: i32,
    pub warehouse_id: Option<i64>,
    pub mapped_location_id: Option<i64>,
    #[sea_orm(default_value = "0")]
    pub pos_x: i32,
    #[sea_orm(default_value = "0")]
    pub pos_y: i32,
    #[sea_orm(default_value = "0")]
    pub rotation: i32,
    #[sea_orm(default_value = "0")]
    pub visual_width: i32,
    #[sea_orm(default_value = "0")]
    pub visual_height: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::WarehouseId",
        to = "super::location::Column::Id"
    )]
    Warehouse,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::MappedLocationId",
        to = "super::location::Column::Id"
    )]
    MappedLocation,
}

impl ActiveModelBehavior for ActiveModel {}
