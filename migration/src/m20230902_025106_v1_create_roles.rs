use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (Role, Role) = (Role::Schema, Role::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: Role = Role::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TABLE)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Role::Id)
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
                    .col(ColumnDef::new(Role::Code).string().not_null().unique_key())
                    .col(ColumnDef::new(Role::Name).string().not_null())
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .name("idx_roles_code")
                    .col(Role::Code)
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .name("idx_roles_name")
                    .col(Role::Name)
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
pub enum Role {
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "roles")]
    Table,
    Id,
    Code,
    Name,
}
