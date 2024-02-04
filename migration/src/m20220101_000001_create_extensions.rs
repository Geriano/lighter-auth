use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(feature = "postgres")]
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .await?;

        #[cfg(feature = "postgres")]
        manager
            .get_connection()
            .execute_unprepared("CREATE SCHEMA IF NOT EXISTS v1")
            .await?;

        #[cfg(feature = "sqlite")]
        manager
            .get_connection()
            .execute_unprepared("PRAGMA foreign_keys = ON")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(feature = "postgres")]
        manager
            .get_connection()
            .execute_unprepared("DROP EXTENSION IF EXISTS \"uuid-ossp\"")
            .await?;

        #[cfg(feature = "postgres")]
        manager
            .get_connection()
            .execute_unprepared("DROP SCHEMA IF EXISTS v1 CASCADE")
            .await?;

        #[cfg(feature = "sqlite")]
        manager
            .get_connection()
            .execute_unprepared("PRAGMA foreign_keys = OFF")
            .await?;

        Ok(())
    }
}
