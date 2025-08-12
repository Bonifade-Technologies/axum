use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert timestamp without time zone -> timestamp with time zone
        let stmt = r#"
            ALTER TABLE users
            ALTER COLUMN created_at TYPE timestamptz USING created_at AT TIME ZONE 'UTC',
            ALTER COLUMN updated_at TYPE timestamptz USING updated_at AT TIME ZONE 'UTC',
            ALTER COLUMN deleted_at TYPE timestamptz USING deleted_at AT TIME ZONE 'UTC';
        "#;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DbBackend::Postgres,
                stmt.to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to timestamp without time zone
        let stmt = r#"
            ALTER TABLE users
            ALTER COLUMN created_at TYPE timestamp USING created_at AT TIME ZONE 'UTC',
            ALTER COLUMN updated_at TYPE timestamp USING updated_at AT TIME ZONE 'UTC',
            ALTER COLUMN deleted_at TYPE timestamp USING deleted_at AT TIME ZONE 'UTC';
        "#;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DbBackend::Postgres,
                stmt.to_string(),
            ))
            .await?;

        Ok(())
    }
}
