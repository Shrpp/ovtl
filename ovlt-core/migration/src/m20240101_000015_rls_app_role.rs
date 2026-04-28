use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000015_rls_app_role"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                // Create a non-superuser role the app switches into for tenant transactions.
                // Superusers bypass RLS even with FORCE; this role does not.
                "DO $$ BEGIN
                    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ovlt_rls') THEN
                        CREATE ROLE ovlt_rls NOLOGIN;
                    END IF;
                END $$;

                GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO ovlt_rls;
                GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ovlt_rls;

                ALTER DEFAULT PRIVILEGES IN SCHEMA public
                    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ovlt_rls;
                ALTER DEFAULT PRIVILEGES IN SCHEMA public
                    GRANT USAGE, SELECT ON SEQUENCES TO ovlt_rls;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP ROLE IF EXISTS ovlt_rls;")
            .await?;
        Ok(())
    }
}
