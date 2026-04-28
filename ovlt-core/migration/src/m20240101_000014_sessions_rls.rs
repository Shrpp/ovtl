use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000014_sessions_rls"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
                ALTER TABLE sessions FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS tenant_isolation ON sessions;
                CREATE POLICY tenant_isolation ON sessions
                    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP POLICY IF EXISTS tenant_isolation ON sessions;
                ALTER TABLE sessions DISABLE ROW LEVEL SECURITY;",
            )
            .await?;
        Ok(())
    }
}
