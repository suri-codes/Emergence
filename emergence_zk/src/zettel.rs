use std::{
    fmt::Display,
    fs::{File, OpenOptions, read_to_string},
    path::PathBuf,
};

use chrono::Local;
use nanoid::nanoid;

use crate::{FrontMatter, Tag, ZkError, ZkResult};

pub struct ZettelId(String);

impl ZettelId {
    fn new() -> Self {
        ZettelId(nanoid!())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ZettelId {
    fn from(value: &str) -> Self {
        ZettelId(value.to_owned())
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_owned())
    }
}

pub struct Zettel {
    path: PathBuf,
    id: ZettelId,
    front_matter: FrontMatter,
    content: String,
}

impl TryFrom<PathBuf> for Zettel {
    type Error = ZkError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let (front_matter, content) = FrontMatter::extract_from_file(&value)?;

        let id: ZettelId = value
            .file_name()
            .ok_or(ZkError::ParseError("Invalid File Name!"))?
            .to_str()
            .ok_or(ZkError::ParseError(
                "File Name cannot be translated into str!",
            ))?
            .into();

        Ok(Zettel {
            path: value,
            id,
            front_matter,
            content,
        })
    }
}

pub struct ZettelBuilder {
    inner: Zettel,
}

impl ZettelBuilder {
    pub fn new(mut project_root: PathBuf) -> Self {
        let id = ZettelId::new();

        let zettel_path = {
            project_root.push(id.as_str());
            project_root
        };

        let front_matter = FrontMatter::new("", Local::now().naive_local(), vec![]);

        ZettelBuilder {
            inner: Zettel {
                id,
                path: zettel_path,
                front_matter,
                content: "".to_owned(),
            },
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.front_matter.name = name;
        self
    }

    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.inner.front_matter.tags.push(tag);
        self
    }

    pub fn content(mut self, content: String) -> Self {
        self.inner.content = content;

        self
    }

    pub fn build(mut self) -> ZkResult<Zettel> {
        let now = Local::now().naive_local();

        // set created at to build time
        self.inner.front_matter.created_at = now;

        OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&self.inner.path)?;

        Ok(self.inner)
    }
}
