use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::ZkError;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct ZettelId(String);

const ALPHABET: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

impl ZettelId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ZettelId {
    fn default() -> Self {
        ZettelId(nanoid!(5, &ALPHABET))
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

impl TryFrom<PathBuf> for ZettelId {
    type Error = ZkError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path = value.as_path();
        path.try_into()
    }
}

impl TryFrom<&Path> for ZettelId {
    type Error = ZkError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let extension =
            value
                .extension()
                .and_then(|ext| ext.to_str())
                .ok_or(ZkError::ParseError(
                    "Unable to turn file extension into string".to_owned(),
                ))?;

        if extension != "md" {
            return Err(ZkError::ParseError(format!(
                "Wrong extension: {extension}, expected .md"
            )));
        }

        let id: ZettelId = value
            .file_name()
            .ok_or(ZkError::ParseError("Invalid File Name!".to_owned()))?
            .to_str()
            .ok_or(ZkError::ParseError(
                "File Name cannot be translated into str!".to_owned(),
            ))?
            .strip_suffix(".md")
            .expect("we statically verify this right above")
            .into();

        Ok(id)
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_owned())
    }
}
