use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE users FORCE ROW LEVEL SECURITY;
                 ALTER TABLE refresh_tokens FORCE ROW LEVEL SECURITY;
                 ALTER TABLE oauth_accounts FORCE ROW LEVEL SECURITY;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE users NO FORCE ROW LEVEL SECURITY;
                 ALTER TABLE refresh_tokens NO FORCE ROW LEVEL SECURITY;
                 ALTER TABLE oauth_accounts NO FORCE ROW LEVEL SECURITY;",
            )
            .await?;
        Ok(())
    }
}
