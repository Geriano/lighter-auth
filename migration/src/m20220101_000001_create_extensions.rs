use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(feature = "sqlite")]
        _manager
            .get_connection()
            .execute_unprepared("PRAGMA foreign_keys = ON")
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(feature = "sqlite")]
        _manager
            .get_connection()
            .execute_unprepared("PRAGMA foreign_keys = OFF")
            .await?;

        Ok(())
    }
}
