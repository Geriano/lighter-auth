use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024725_v1_create_users::{User, TABLE as USER_TABLE},
    m20230902_024928_v1_create_permissions::{Permission, TABLE as PERMISSION_TABLE},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (PermissionUser, PermissionUser) = (PermissionUser::Schema, PermissionUser::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: PermissionUser = PermissionUser::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(any(feature = "postgres", feature = "sqlite"))]
        manager
            .create_table(
                Table::create()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(PermissionUser::Id)
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
                        ColumnDef::new(PermissionUser::PermissionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PermissionUser::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, PermissionUser::PermissionId)
                            .to(PERMISSION_TABLE, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, PermissionUser::UserId)
                            .to(USER_TABLE, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(PermissionUser::PermissionId)
                    .name("idx_permission_user_permission_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(PermissionUser::UserId)
                    .name("idx_permission_user_user_id")
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
pub enum PermissionUser {
    #[cfg(feature = "postgres")]
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "permission_user")]
    Table,
    Id,
    PermissionId,
    UserId,
}
