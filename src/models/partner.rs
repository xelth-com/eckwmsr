use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::odoo_types::OdooString;

/// ResPartner — UUID-native partner entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "res_partner")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub street: OdooString,
    pub street2: OdooString,
    pub zip: OdooString,
    pub city: OdooString,
    pub state_id: Option<Uuid>,
    pub country_id: Option<Uuid>,
    pub phone: OdooString,
    pub email: OdooString,
    pub vat: OdooString,
    pub company_type: String,
    pub is_company: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
