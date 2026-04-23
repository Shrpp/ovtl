use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000005_security_hardening"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // FORCE RLS on users and oauth_accounts so even the table owner is subject to policies.
        // refresh_tokens is intentionally excluded — the background cleanup task needs cross-tenant
        // access without a transaction context.
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE users FORCE ROW LEVEL SECURITY")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE oauth_accounts FORCE ROW LEVEL SECURITY")
            .await?;

        // Track failed login attempts per (tenant, email) for account lockout.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS login_attempts (
                    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id    UUID        NOT NULL,
                    email_lookup TEXT        NOT NULL,
                    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now()
                )",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_login_attempts_lookup
                 ON login_attempts (tenant_id, email_lookup, attempted_at)",
            )
            .await?;

        // Append-only audit trail — no RLS, written by the server.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS audit_log (
                    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id  UUID        NOT NULL,
                    user_id    UUID,
                    action     TEXT        NOT NULL,
                    ip         TEXT,
                    metadata   TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                )",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_audit_log_tenant
                 ON audit_log (tenant_id, created_at)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE oauth_accounts NO FORCE ROW LEVEL SECURITY")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE users NO FORCE ROW LEVEL SECURITY")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS audit_log")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS login_attempts")
            .await?;
        Ok(())
    }
}
