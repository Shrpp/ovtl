use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE oauth_clients
                    ADD COLUMN IF NOT EXISTS access_token_ttl_minutes INT,
                    ADD COLUMN IF NOT EXISTS refresh_token_ttl_days   INT;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE oauth_clients
                    DROP COLUMN IF EXISTS access_token_ttl_minutes,
                    DROP COLUMN IF EXISTS refresh_token_ttl_days;",
            )
            .await?;
        Ok(())
    }
}
