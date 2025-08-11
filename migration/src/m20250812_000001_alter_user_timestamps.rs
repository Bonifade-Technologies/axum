use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert naive timestamps to timestamptz assuming existing values are UTC.
        // USING created_at AT TIME ZONE 'UTC' keeps the stored moment the same.
        let conn = manager.get_connection();
        let stmts = [
            "ALTER TABLE users ALTER COLUMN created_at TYPE timestamptz USING created_at AT TIME ZONE 'UTC'",
            "ALTER TABLE users ALTER COLUMN updated_at TYPE timestamptz USING updated_at AT TIME ZONE 'UTC'",
            "ALTER TABLE users ALTER COLUMN deleted_at TYPE timestamptz USING deleted_at AT TIME ZONE 'UTC'",
        ];
        for sql in stmts {
            conn.execute(Statement::from_string(DbBackend::Postgres, sql.to_string()))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to timestamp without time zone, preserving UTC values
        let conn = manager.get_connection();
        let stmts = [
            "ALTER TABLE users ALTER COLUMN created_at TYPE timestamp USING created_at AT TIME ZONE 'UTC'",
            "ALTER TABLE users ALTER COLUMN updated_at TYPE timestamp USING updated_at AT TIME ZONE 'UTC'",
            "ALTER TABLE users ALTER COLUMN deleted_at TYPE timestamp USING deleted_at AT TIME ZONE 'UTC'",
        ];
        for sql in stmts {
            conn.execute(Statement::from_string(DbBackend::Postgres, sql.to_string()))
                .await?;
        }
        Ok(())
    }
}
