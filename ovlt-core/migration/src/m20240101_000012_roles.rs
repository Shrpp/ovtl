use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS roles (
                    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id   UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                    name        TEXT        NOT NULL,
                    description TEXT        NOT NULL DEFAULT '',
                    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                    UNIQUE(tenant_id, name)
                );
                CREATE INDEX IF NOT EXISTS idx_roles_tenant ON roles(tenant_id);
                ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
                ALTER TABLE roles FORCE ROW LEVEL SECURITY;
                CREATE POLICY tenant_isolation ON roles
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

                CREATE TABLE IF NOT EXISTS user_roles (
                    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    role_id     UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                    tenant_id   UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                    PRIMARY KEY (user_id, role_id)
                );
                CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);
                CREATE INDEX IF NOT EXISTS idx_user_roles_tenant ON user_roles(tenant_id);
                ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;
                ALTER TABLE user_roles FORCE ROW LEVEL SECURITY;
                CREATE POLICY tenant_isolation ON user_roles
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS user_roles;
                 DROP TABLE IF EXISTS roles;",
            )
            .await?;
        Ok(())
    }
}
