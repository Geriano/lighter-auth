use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024928_v1_create_permissions::Permission, m20230902_025106_v1_create_roles::Role,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PermissionRole::Table)
                    .col(
                        ColumnDef::new(PermissionRole::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PermissionRole::PermissionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PermissionRole::RoleId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PermissionRole::Table, PermissionRole::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PermissionRole::Table, PermissionRole::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PermissionRole::Table)
                    .col(PermissionRole::PermissionId)
                    .name("idx_permission_role_permission_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PermissionRole::Table)
                    .col(PermissionRole::RoleId)
                    .name("idx_permission_role_role_id")
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(PermissionRole::Table)
                    .take(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum PermissionRole {
    #[sea_orm(iden = "permission_role")]
    Table,
    Id,
    PermissionId,
    RoleId,
}
