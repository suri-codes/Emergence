use std::path::PathBuf;

use crate::{EmergenceDb, ZkResult};
pub struct Workspace {
    pub root: PathBuf,
    pub db: EmergenceDb,
}

impl Workspace {
    pub async fn init(root: impl Into<PathBuf>) -> ZkResult<Self> {
        let root = root.into();
        let db = EmergenceDb::connect(&root).await?;
        Ok(Self { root, db })
    }
}
