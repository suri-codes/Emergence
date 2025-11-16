use sea_orm_migration::{prelude::*, schema::*};

use crate::m20251104_023917_create_tag_table::Tag;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Zettel::Table)
                    .if_not_exists()
                    .col(pk_auto(Zettel::Id).not_null())
                    .col(string(Zettel::Nanoid).unique_key().not_null())
                    .col(string(Zettel::Title).not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Zettel::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Zettel {
    Table,
    Id,
    Nanoid,
    Title,
}
