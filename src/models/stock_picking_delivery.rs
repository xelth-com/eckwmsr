use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_picking_delivery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub picking_id: Option<i64>,
    pub carrier_id: Option<i64>,
    #[sea_orm(index)]
    pub tracking_number: String,
    pub carrier_price: f64,
    pub currency: String,
    #[sea_orm(default_value = "draft")]
    pub status: String,
    #[sea_orm(column_type = "Text")]
    pub error_message: String,
    pub label_url: String,
    pub raw_response: String,
    pub shipped_at: Option<DateTimeUtc>,
    pub delivered_at: Option<DateTimeUtc>,
    pub last_activity_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
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
        belongs_to = "super::delivery_carrier::Entity",
        from = "Column::CarrierId",
        to = "super::delivery_carrier::Column::Id"
    )]
    Carrier,
}
impl ActiveModelBehavior for ActiveModel {}
