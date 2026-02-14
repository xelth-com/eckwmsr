use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_auths")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub role: String,
    #[sea_orm(column_name = "user_type")]
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[sea_orm(column_name = "google_id")]
    #[serde(rename = "googleId", skip_serializing_if = "Option::is_none")]
    pub google_id: Option<String>,
    #[serde(skip_serializing)]
    pub pin: String,
    #[sea_orm(column_name = "is_active")]
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[sea_orm(column_name = "last_login")]
    #[serde(rename = "lastLogin", skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTimeUtc>,
    #[sea_orm(column_name = "failed_login_attempts")]
    #[serde(skip_serializing)]
    pub failed_login_attempts: i32,
    #[sea_orm(column_name = "preferred_language")]
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: String,
    #[sea_orm(column_name = "created_at")]
    #[serde(rename = "createdAt")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_name = "updated_at")]
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(column_name = "deleted_at")]
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
