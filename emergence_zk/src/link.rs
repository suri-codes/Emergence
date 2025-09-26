use std::path::{Path, PathBuf};

use crate::{ZettelId, ZkError};

pub struct Link {
    pub source: ZettelId,
    pub dest: ZettelId,
}

impl Link {
    pub fn new(source: impl Into<ZettelId>, dest: impl Into<ZettelId>) -> Self {
        Self {
            source: source.into(),
            dest: dest.into(),
        }
    }
}
