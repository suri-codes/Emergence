use std::path::PathBuf;

use crate::{ZettelId, ZkError};

pub struct Link {
    source: ZettelId,
    dest: ZettelId,
}

impl Link {
    pub fn new(source: ZettelId, dest: ZettelId) -> Self {
        Self { source, dest }
    }
}

impl TryFrom<&PathBuf> for Link {
    type Error = ZkError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let source_id: ZettelId = value.try_into()?;
        

        todo!()
    }
}
