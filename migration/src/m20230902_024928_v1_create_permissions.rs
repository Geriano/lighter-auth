use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (Permission, Permission) = (Permission::Schema, Permission::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: Permission = Permission::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(Permission::Id)
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
                        ColumnDef::new(Permission::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Permission::Name).string().not_null())
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(Permission::Code)
                    .name("idx_permissions_code")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(Permission::Name)
                    .name("idx_permissions_name")
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(TABLE).take())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Permission {
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "permissions")]
    Table,
    Id,
    Code,
    Name,
}
