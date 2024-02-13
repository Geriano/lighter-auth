use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024725_v1_create_users::{User, TABLE as USER_TABLE},
    m20230902_025106_v1_create_roles::{Role, TABLE as ROLE_TABLE},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (RoleUser, RoleUser) = (RoleUser::Schema, RoleUser::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: RoleUser = RoleUser::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(RoleUser::Id)
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
                    .col(ColumnDef::new(RoleUser::RoleId).uuid().not_null())
                    .col(ColumnDef::new(RoleUser::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, RoleUser::RoleId)
                            .to(ROLE_TABLE, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, RoleUser::UserId)
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
                    .col(RoleUser::RoleId)
                    .name("idx_role_user_role_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(RoleUser::UserId)
                    .name("idx_role_user_user_id")
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
pub enum RoleUser {
    #[cfg(feature = "postgres")]
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "role_user")]
    Table,
    Id,
    RoleId,
    UserId,
}
