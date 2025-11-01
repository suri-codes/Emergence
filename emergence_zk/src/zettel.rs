use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{FrontMatter, Metadata, Tag, ZettelId, ZkResult};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Zettel {
    pub path: PathBuf,
    pub id: ZettelId,
    pub front_matter: FrontMatter,
    pub tags: Vec<Tag>,
    pub content: String,
}

impl Zettel {
    pub fn new(
        id: ZettelId,
        path: PathBuf,
        front_matter: FrontMatter,
        tags: Vec<Tag>,
        content: String,
    ) -> Self {
        Self {
            path,
            id,
            front_matter,
            tags,
            content,
        }
    }
}

pub struct ZettelBuilder {
    inner: Zettel,
}

impl ZettelBuilder {
    pub fn new(metadata: &Metadata) -> Self {
        let id = ZettelId::default();

        let zettel_path = {
            let mut project_root = metadata.project_root.clone();
            project_root.push([id.as_str(), ".md"].join(""));
            project_root
        };

        let front_matter = FrontMatter::new("", Local::now().naive_local(), Vec::<String>::new());

        ZettelBuilder {
            inner: Zettel {
                id,
                path: zettel_path,
                front_matter,
                content: "".to_owned(),
                tags: Vec::new(),
            },
        }
    }

    // methods for mutating inner state

    pub fn name(&mut self, name: impl Into<String>) {
        self.inner.front_matter.name = name.into();
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.inner.tags.push(tag);
    }

    pub fn content(&mut self, content: impl Into<String>) {
        self.inner.content = content.into();
    }

    // methods for builder pattern

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.inner.front_matter.name = name.into();
        self
    }

    pub fn with_additional_tag(mut self, tag: Tag) -> Self {
        self.inner.tags.push(tag);
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.inner.content = content.into();

        self
    }

    pub fn build(mut self) -> ZkResult<Zettel> {
        let now = Local::now().naive_local();

        // set created_at to build time
        self.inner.front_matter.created_at = now;

        let mut f = OpenOptions::new()
            .create_new(true)
            .read(true)
            .append(true)
            .open(&self.inner.path)?;

        writeln!(f, "{}", self.inner.front_matter)?;
        writeln!(f, "{}", self.inner.content)?;

        Ok(self.inner)
    }
}
