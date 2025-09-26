use std::{fmt::Display, path::Path};

use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::ZkError;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct ZettelId(String);

impl ZettelId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ZettelId {
    fn default() -> Self {
        ZettelId(nanoid!())
    }
}

impl From<&str> for ZettelId {
    fn from(value: &str) -> Self {
        ZettelId(value.to_owned())
    }
}

impl From<&ZettelId> for ZettelId {
    fn from(value: &ZettelId) -> Self {
        value.clone()
    }
}

impl TryFrom<&Path> for ZettelId {
    type Error = ZkError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let id: ZettelId = value
            .file_name()
            .ok_or(ZkError::ParseError("Invalid File Name!".to_owned()))?
            .to_str()
            .ok_or(ZkError::ParseError(
                "File Name cannot be translated into str!".to_owned(),
            ))?
            .into();

        Ok(id)
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_owned())
    }
}
