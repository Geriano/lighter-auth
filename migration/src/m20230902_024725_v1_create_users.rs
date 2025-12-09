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
                    .table(User::Table)
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(ColumnDef::new(User::Email).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(User::EmailVerifiedAt)
                            .timestamp()
                            .null()
                            .default(None as Option<String>),
                    )
                    .col(
                        ColumnDef::new(User::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::Password).string().not_null())
                    .col(
                        ColumnDef::new(User::ProfilePhotoId)
                            .string()
                            .null()
                            .default(None as Option<String>),
                    )
                    .col(ColumnDef::new(User::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(User::UpdatedAt).timestamp().not_null())
                    .col(
                        ColumnDef::new(User::DeletedAt)
                            .timestamp()
                            .null()
                            .default(None as Option<String>),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::Email)
                    .name("idx_users_email")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::EmailVerifiedAt)
                    .name("idx_users_email_verified_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::Username)
                    .name("idx_users_username")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::CreatedAt)
                    .name("idx_users_created_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::UpdatedAt)
                    .name("idx_users_updated_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::DeletedAt)
                    .name("idx_users_deleted_at")
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(User::Table).take())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    Name,
    Email,
    EmailVerifiedAt,
    Username,
    Password,
    ProfilePhotoId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
