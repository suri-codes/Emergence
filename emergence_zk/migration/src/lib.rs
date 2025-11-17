pub use sea_orm_migration::prelude::*;

pub(crate) mod m20251104_023917_create_tag_table;
pub(crate) mod m20251104_024116_create_zettel_table;
mod m20251104_050736_create_zettel_tag_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251104_023917_create_tag_table::Migration),
            Box::new(m20251104_024116_create_zettel_table::Migration),
            Box::new(m20251104_050736_create_zettel_tag_table::Migration),
        ]
    }
}
