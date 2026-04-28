use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000007_authorization_codes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS authorization_codes (
                    code            TEXT        PRIMARY KEY,
                    tenant_id       UUID        NOT NULL,
                    client_id       TEXT        NOT NULL REFERENCES oauth_clients(client_id),
                    user_id         UUID        NOT NULL,
                    redirect_uri    TEXT        NOT NULL,
                    scopes          JSONB       NOT NULL DEFAULT '[]',
                    code_challenge  TEXT        NOT NULL,
                    nonce           TEXT,
                    expires_at      TIMESTAMPTZ NOT NULL,
                    used_at         TIMESTAMPTZ
                );
                CREATE INDEX IF NOT EXISTS idx_auth_codes_expires ON authorization_codes (expires_at);
                CREATE INDEX IF NOT EXISTS idx_auth_codes_client ON authorization_codes (client_id)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS authorization_codes")
            .await?;
        Ok(())
    }
}
