use crate::m20251104_023917_create_tag_table::Tag;
use crate::m20251104_024116_create_zettel_table::Zettel;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(ZettelTag::Table)
                    .if_not_exists()
                    .col(string(ZettelTag::ZettelNanoId).not_null())
                    .col(string(ZettelTag::TagNanoId).not_null())
                    .primary_key(
                        Index::create()
                            .col(ZettelTag::ZettelNanoId)
                            .col(ZettelTag::TagNanoId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-zettel_tag-zettel_nano_id")
                            .from(ZettelTag::Table, ZettelTag::ZettelNanoId)
                            .to(Zettel::Table, Zettel::Nanoid)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-zettel_tag-tag_nano_id")
                            .from(ZettelTag::Table, ZettelTag::TagNanoId)
                            .to(Tag::Table, Tag::Nanoid)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(ZettelTag::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ZettelTag {
    Table,
    ZettelNanoId,
    TagNanoId,
}
