use std::path::PathBuf;

use sea_orm::{Database, DatabaseConnection};

use crate::ZkResult;

pub mod entities;
pub use sea_orm::entity;

use migration::{Migrator, MigratorTrait};

#[derive(Debug, Clone)]
pub struct EmergenceDb {
    inner: DatabaseConnection,
}

impl AsRef<DatabaseConnection> for EmergenceDb {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.inner
    }
}

impl EmergenceDb {
    pub async fn connect(root: impl Into<PathBuf>) -> ZkResult<Self> {
        let root_folder = root.into();
        let path = format!(
            "{}/.emergence/emergence.sqlite",
            root_folder.canonicalize()?.as_path().to_string_lossy()
        );

        let db: DatabaseConnection =
            Database::connect(format!("sqlite://{}?mode=rwc", path)).await?;

        // apply all migrations
        Migrator::up(&db, None).await?;
        // synchronizes database schema with entity definitions

        Ok(Self { inner: db })
    }

    
}
