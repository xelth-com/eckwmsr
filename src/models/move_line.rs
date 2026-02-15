use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::odoo_types::OdooString;

/// StockMoveLine mirrors Odoo 'stock.move.line' (Move Details)
/// Matches Go's `StockMoveLine` struct from `internal/models/stock.go`
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_move_line")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub picking_id: i64,
    pub product_id: i64,
    pub qty_done: f64,
    pub location_id: i64,
    pub location_dest_id: i64,
    pub package_id: Option<i64>,
    pub result_package_id: Option<i64>,
    pub lot_id: Option<i64>,
    pub state: String,
    pub reference: OdooString,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::picking::Entity",
        from = "Column::PickingId",
        to = "super::picking::Column::Id"
    )]
    Picking,
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::LocationId",
        to = "super::location::Column::Id"
    )]
    Location,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::LocationDestId",
        to = "super::location::Column::Id"
    )]
    LocationDest,
}

impl ActiveModelBehavior for ActiveModel {}
