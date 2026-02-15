use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "delivery_tracking")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(index)]
    pub picking_delivery_id: i64,
    pub timestamp: DateTimeUtc,
    pub status: String,
    pub status_code: String,
    pub location: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stock_picking_delivery::Entity",
        from = "Column::PickingDeliveryId",
        to = "super::stock_picking_delivery::Column::Id"
    )]
    StockPickingDelivery,
}
impl ActiveModelBehavior for ActiveModel {}
