use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::odoo_types::OdooString;

/// StockPicking — UUID-native transfer order entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_picking")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub state: String,
    pub location_id: Uuid,
    pub location_dest_id: Uuid,
    pub scheduled_date: DateTimeUtc,
    pub origin: OdooString,
    pub priority: String,
    pub picking_type_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub date_done: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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
    #[sea_orm(
        belongs_to = "super::partner::Entity",
        from = "Column::PartnerId",
        to = "super::partner::Column::Id"
    )]
    Partner,
}

impl ActiveModelBehavior for ActiveModel {}
