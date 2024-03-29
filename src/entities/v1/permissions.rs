//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[cfg_attr(feature = "postgres", sea_orm(schema_name = "v1"))]
#[sea_orm(table_name = "permissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub code: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::permission_role::Entity")]
    PermissionRole,
    #[sea_orm(has_many = "super::permission_user::Entity")]
    PermissionUser,
}

impl Related<super::permission_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PermissionRole.def()
    }
}

impl Related<super::permission_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PermissionUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
