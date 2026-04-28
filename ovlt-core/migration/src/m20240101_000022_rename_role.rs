use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$
                BEGIN
                    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ovtl_rls') THEN
                        ALTER ROLE ovtl_rls RENAME TO ovlt_rls;
                    END IF;
                END
                $$;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$
                BEGIN
                    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ovlt_rls') THEN
                        ALTER ROLE ovlt_rls RENAME TO ovtl_rls;
                    END IF;
                END
                $$;",
            )
            .await?;
        Ok(())
    }
}
