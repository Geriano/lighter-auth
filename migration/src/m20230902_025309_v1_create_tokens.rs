use sea_orm_migration::prelude::*;

use crate::m20230902_024725_v1_create_users::{User, TABLE as USER_TABLE};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[cfg(feature = "postgres")]
const TABLE: (Token, Token) = (Token::Schema, Token::Table);
#[cfg(not(feature = "postgres"))]
const TABLE: Token = Token::Table;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(any(feature = "postgres", feature = "sqlite"))]
        manager
            .create_table(
                Table::create()
                    .table(TABLE)
                    .col(
                        ColumnDef::new(Token::Id)
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
                    .col(ColumnDef::new(Token::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Token::ExpiredAt)
                            .timestamp()
                            .null()
                            .extra("default null"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TABLE, Token::UserId)
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
                    .col(Token::UserId)
                    .name("idx_token_user_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TABLE)
                    .col(Token::ExpiredAt)
                    .name("idx_token_expired_at")
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
enum Token {
    #[cfg(feature = "postgres")]
    #[sea_orm(iden = "v1")]
    Schema,
    #[sea_orm(iden = "tokens")]
    Table,
    Id,
    UserId,
    ExpiredAt,
}
