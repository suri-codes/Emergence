use std::io::Write;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{FrontMatter, Tag, ZettelId, ZkError, ZkResult};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Zettel {
    pub path: PathBuf,
    pub id: ZettelId,
    pub meta: FrontMatter,
    pub content: String,
}

impl TryFrom<&Path> for Zettel {
    type Error = ZkError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let (front_matter, content) = FrontMatter::extract_from_file(value)?;

        let id: ZettelId = value.try_into()?;

        Ok(Zettel {
            path: value.to_path_buf(),
            id,
            meta: front_matter,
            content,
        })
    }
}

pub struct ZettelBuilder {
    inner: Zettel,
}

impl ZettelBuilder {
    pub fn new(mut project_root: PathBuf) -> Self {
        let id = ZettelId::default();

        let zettel_path = {
            project_root.push([id.as_str(), ".md"].join(""));
            project_root
        };

        let front_matter = FrontMatter::new("", Local::now().naive_local(), vec![]);

        ZettelBuilder {
            inner: Zettel {
                id,
                path: zettel_path,
                meta: front_matter,
                content: "".to_owned(),
            },
        }
    }

    // methods for mutating inner state

    pub fn name(&mut self, name: impl Into<String>) {
        self.inner.meta.name = name.into();
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.inner.meta.tags.push(tag);
    }

    pub fn content(&mut self, content: impl Into<String>) {
        self.inner.content = content.into();
    }

    // methods for builder pattern

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.inner.meta.name = name.into();
        self
    }

    pub fn with_additional_tag(mut self, tag: Tag) -> Self {
        self.inner.meta.tags.push(tag);
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.inner.content = content.into();

        self
    }

    pub fn build(mut self) -> ZkResult<Zettel> {
        let now = Local::now().naive_local();

        // set created_at to build time
        self.inner.meta.created_at = now;

        let mut f = OpenOptions::new()
            .create_new(true)
            .read(true)
            .append(true)
            .open(&self.inner.path)?;

        writeln!(f, "{}", self.inner.meta)?;
        writeln!(f, "{}", self.inner.content)?;

        Ok(self.inner)
    }
}
