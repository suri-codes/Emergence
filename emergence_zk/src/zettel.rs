use std::{fmt::Display, fs::OpenOptions, io::Write, path::PathBuf};

use chrono::Local;
use nanoid::nanoid;

use crate::{FrontMatter, Tag, ZkError, ZkResult};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

impl TryFrom<&PathBuf> for ZettelId {
    type Error = ZkError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Zettel {
    path: PathBuf,
    id: ZettelId,
    meta: FrontMatter,
    content: String,
}

impl TryFrom<&PathBuf> for Zettel {
    type Error = ZkError;
    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let (front_matter, content) = FrontMatter::extract_from_file(&value)?;

        let id: ZettelId = value.try_into()?;

        Ok(Zettel {
            path: value.clone(),
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
        let id = ZettelId::new();

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

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.inner.meta.name = name.into();
        self
    }

    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.inner.meta.tags.push(tag);
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
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
