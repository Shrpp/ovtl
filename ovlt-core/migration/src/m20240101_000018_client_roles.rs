use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS client_roles (
                    oauth_client_id UUID        NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
                    role_id         UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                    tenant_id       UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                    assigned_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
                    PRIMARY KEY (oauth_client_id, role_id)
                );
                CREATE INDEX IF NOT EXISTS idx_client_roles_client ON client_roles(oauth_client_id);
                CREATE INDEX IF NOT EXISTS idx_client_roles_tenant ON client_roles(tenant_id);
                ALTER TABLE client_roles ENABLE ROW LEVEL SECURITY;
                ALTER TABLE client_roles FORCE ROW LEVEL SECURITY;
                CREATE POLICY tenant_isolation ON client_roles
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS client_roles;")
            .await?;
        Ok(())
    }
}
