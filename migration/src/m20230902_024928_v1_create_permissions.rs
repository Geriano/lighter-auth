use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Permission::Table)
                    .col(
                        ColumnDef::new(Permission::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
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
                    .table(Permission::Table)
                    .col(Permission::Code)
                    .name("idx_permissions_code")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Permission::Table)
                    .col(Permission::Name)
                    .name("idx_permissions_name")
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Permission::Table).take())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Permission {
    #[sea_orm(iden = "permissions")]
    Table,
    Id,
    Code,
    Name,
}
