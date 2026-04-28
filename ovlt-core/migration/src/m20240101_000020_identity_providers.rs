use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS identity_providers (
                    id                 UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id          UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                    provider           TEXT        NOT NULL,
                    client_id          TEXT        NOT NULL,
                    client_secret_enc  TEXT        NOT NULL,
                    redirect_url       TEXT        NOT NULL,
                    scopes             JSONB       NOT NULL DEFAULT '[\"openid\",\"email\",\"profile\"]',
                    enabled            BOOL        NOT NULL DEFAULT true,
                    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
                    UNIQUE (tenant_id, provider)
                );
                CREATE INDEX IF NOT EXISTS idx_idp_tenant ON identity_providers(tenant_id);
                ALTER TABLE identity_providers ENABLE ROW LEVEL SECURITY;
                ALTER TABLE identity_providers FORCE ROW LEVEL SECURITY;
                CREATE POLICY tenant_isolation ON identity_providers
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS identity_providers;")
            .await?;
        Ok(())
    }
}
