use sea_orm_migration::prelude::*;

use crate::{m20230902_024725_v1_create_users::User, m20230902_025106_v1_create_roles::Role};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RoleUser::Table)
                    .col(ColumnDef::new(RoleUser::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(RoleUser::RoleId).uuid().not_null())
                    .col(ColumnDef::new(RoleUser::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(RoleUser::Table, RoleUser::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RoleUser::Table, RoleUser::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(RoleUser::Table)
                    .col(RoleUser::RoleId)
                    .name("idx_role_user_role_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(RoleUser::Table)
                    .col(RoleUser::UserId)
                    .name("idx_role_user_user_id")
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(RoleUser::Table).take())
            .await
    }
}

#[derive(DeriveIden)]
pub enum RoleUser {
    #[sea_orm(iden = "role_user")]
    Table,
    Id,
    RoleId,
    UserId,
}
