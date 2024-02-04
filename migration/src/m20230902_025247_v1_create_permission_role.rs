use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024928_v1_create_permissions::{Permission, TABLE as PERMISSION_TABLE},
    m20230902_025106_v1_create_roles::{Role, TABLE as ROLE_TABLE},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (PermissionRole, PermissionRole) = (PermissionRole::Schema, PermissionRole::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: PermissionRole = PermissionRole::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(PermissionRole::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra(
                                #[cfg(feature = "postgres")]
                                "DEFAULT uuid_generate_v4()",
                                #[cfg(feature = "sqlite")]
                                "DEFAULT (hex(randomblob(16)))",
                            ),
                    )
                    .col(
                        ColumnDef::new(PermissionRole::PermissionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PermissionRole::RoleId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, PermissionRole::PermissionId)
                            .to(PERMISSION_TABLE, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, PermissionRole::RoleId)
                            .to(ROLE_TABLE, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(PermissionRole::PermissionId)
                    .name("idx_permission_role_permission_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(PermissionRole::RoleId)
                    .name("idx_permission_role_role_id")
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(TABLE).take())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PermissionRole {
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "permission_role")]
    Table,
    Id,
    PermissionId,
    RoleId,
}
