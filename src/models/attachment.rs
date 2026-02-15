use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// EntityAttachment links a FileResource to a business entity (polymorphic).
/// Matches Go's `EntityAttachment` from `internal/models/attachment.go`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "entity_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub id: Uuid,
    pub file_resource_id: Uuid,
    pub res_model: String,
    pub res_id: String,
    #[sea_orm(default_value = "false")]
    pub is_main: bool,
    pub tags: Option<String>,
    pub comment: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::file_resource::Entity",
        from = "Column::FileResourceId",
        to = "super::file_resource::Column::Id"
    )]
    FileResource,
}

impl Related<super::file_resource::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileResource.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
