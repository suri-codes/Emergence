use std::{collections::HashMap, path::PathBuf};

use crate::{Tag, TagMap, ZkError, ZkResult};

pub struct Metadata {
    pub tags: TagMap,
    pub project_root: PathBuf,
}

impl TryFrom<PathBuf> for Metadata {
    type Error = ZkError;

    fn try_from(root: PathBuf) -> Result<Self, Self::Error> {
        let _meta_folder = {
            let mut tmp = root.clone();
            tmp.push(".emergence");
            tmp
        };

        // let tag_map = Tag::get_tag_map(meta_folder)?;

        Ok(Self {
            tags: HashMap::new(),
            project_root: root,
        })
    }
}

impl Metadata {
    pub fn parse(root: impl Into<PathBuf>) -> ZkResult<Self> {
        root.into().try_into()
    }
    pub fn lookup_tag_string(&self, _tag_string: &str) -> Option<Tag> {
        // this code is completely wrong
        Some(Tag::new("lol", "wahoo").unwrap())
    }
}
