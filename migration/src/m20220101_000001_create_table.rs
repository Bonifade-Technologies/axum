use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id).not_null().primary_key())
                    .col(string(User::Name).not_null())
                    .col(string(User::Email).unique_key().not_null())
                    .col(number(User::Phone))
                    .col(char(User::Password))
                    .col(string(User::Avatar))
                    .col(timestamp(User::CreatedAt))
                    .col(timestamp(User::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    Phone,
    Password,
    Avatar,
    CreatedAt,
    UpdatedAt,
}
