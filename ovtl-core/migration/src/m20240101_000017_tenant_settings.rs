use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000017_tenant_settings"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS tenant_settings (
                tenant_id                  UUID    PRIMARY KEY,
                lockout_max_attempts       INTEGER NOT NULL DEFAULT 5,
                lockout_window_minutes     INTEGER NOT NULL DEFAULT 15,
                lockout_duration_minutes   INTEGER NOT NULL DEFAULT 15,
                access_token_ttl_minutes   INTEGER NOT NULL DEFAULT 15,
                refresh_token_ttl_days     INTEGER NOT NULL DEFAULT 30,
                allow_public_registration  BOOLEAN NOT NULL DEFAULT true,
                require_email_verified     BOOLEAN NOT NULL DEFAULT false,
                updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS tenant_settings")
            .await?;
        Ok(())
    }
}
