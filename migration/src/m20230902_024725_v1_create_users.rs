use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
pub const TABLE: (User, User) = (User::Schema, User::Table);
#[cfg(not(feature = "postgres"))]
pub const TABLE: User = User::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(any(feature = "postgres", feature = "sqlite"))]
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra(
                                #[cfg(feature = "postgres")]
                                "DEFAULT uuid_generate_v4()",
                                #[cfg(feature = "sqlite")]
                                "DEFAULT (lower(hex(randomblob(16))))",
                            ),
                    )
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
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(
                                #[cfg(feature = "postgres")]
                                "DEFAULT NOW()",
                                #[cfg(feature = "sqlite")]
                                "DEFAULT CURRENT_TIMESTAMP",
                            ),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(
                                #[cfg(feature = "postgres")]
                                "DEFAULT NOW()",
                                #[cfg(feature = "sqlite")]
                                "DEFAULT CURRENT_TIMESTAMP",
                            ),
                    )
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
                    .table(TABLE)
                    .col(User::Email)
                    .name("idx_users_email")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(User::EmailVerifiedAt)
                    .name("idx_users_email_verified_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(User::Username)
                    .name("idx_users_username")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(User::CreatedAt)
                    .name("idx_users_created_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(User::UpdatedAt)
                    .name("idx_users_updated_at")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(User::DeletedAt)
                    .name("idx_users_deleted_at")
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
pub enum User {
    #[cfg(feature = "postgres")]
    #[sea_orm(iden = "v1")]
    Schema,
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
