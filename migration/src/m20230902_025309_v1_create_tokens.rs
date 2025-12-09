use sea_orm_migration::prelude::*;

use crate::m20230902_024725_v1_create_users::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Token::Table)
                    .col(ColumnDef::new(Token::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Token::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Token::ExpiredAt)
                            .timestamp()
                            .null()
                            .extra("default null"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Token::Table, Token::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Token::Table)
                    .col(Token::UserId)
                    .name("idx_token_user_id")
                    .take(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Token::Table)
                    .col(Token::ExpiredAt)
                    .name("idx_token_expired_at")
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Token::Table).take())
            .await
    }
}

#[derive(DeriveIden)]
enum Token {
    #[sea_orm(iden = "tokens")]
    Table,
    Id,
    UserId,
    ExpiredAt,
}
