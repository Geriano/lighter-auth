use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024725_v1_create_users::User, m20230902_024928_v1_create_permissions::Permission,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PermissionUser::Table)
                    .col(
                        ColumnDef::new(PermissionUser::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PermissionUser::PermissionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PermissionUser::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PermissionUser::Table, PermissionUser::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PermissionUser::Table, PermissionUser::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PermissionUser::Table)
                    .col(PermissionUser::PermissionId)
                    .name("idx_permission_user_permission_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PermissionUser::Table)
                    .col(PermissionUser::UserId)
                    .name("idx_permission_user_user_id")
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
                    .table(PermissionUser::Table)
                    .take(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum PermissionUser {
    #[sea_orm(iden = "permission_user")]
    Table,
    Id,
    PermissionId,
    UserId,
}
