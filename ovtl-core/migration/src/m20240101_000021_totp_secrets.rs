use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS totp_secrets (
                    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id   UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                    user_id     UUID        NOT NULL,
                    secret_enc  TEXT        NOT NULL,
                    enabled     BOOL        NOT NULL DEFAULT false,
                    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                    UNIQUE (tenant_id, user_id)
                );
                CREATE INDEX IF NOT EXISTS idx_totp_user ON totp_secrets(user_id);
                ALTER TABLE totp_secrets ENABLE ROW LEVEL SECURITY;
                ALTER TABLE totp_secrets FORCE ROW LEVEL SECURITY;
                CREATE POLICY tenant_isolation ON totp_secrets
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);
                GRANT SELECT, INSERT, UPDATE, DELETE ON totp_secrets TO ovtl_app;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS totp_secrets;")
            .await?;
        Ok(())
    }
}
