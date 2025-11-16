use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use chrono::Local;
use sea_orm::ActiveModelTrait;
use serde::{Deserialize, Serialize};

use crate::{EmergenceDb, FrontMatter, Metadata, Tag, ZettelId, ZkResult, entities, zettel};

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

    pub fn active_model(&self) -> entities::zettel::ActiveModel {
        entities::zettel::ActiveModel {
            nanoid: sea_orm::ActiveValue::Set(self.id.to_string()),
            title: sea_orm::ActiveValue::Set(self.front_matter.title.clone()),
            ..Default::default()
        }
    }
}



pub struct ZettelBuilder {
    inner: Zettel,
}

impl ZettelBuilder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let id = ZettelId::default();

        let zettel_path = {
            let mut project_root = root.into();
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
        self.inner.front_matter.title = name.into();
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.inner.tags.push(tag);
    }

    pub fn content(&mut self, content: impl Into<String>) {
        self.inner.content = content.into();
    }

    // methods for builder pattern

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.inner.front_matter.title = name.into();
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

    pub async fn build(mut self, db: &EmergenceDb) -> ZkResult<Zettel> {
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

        let am = self.inner.active_model();

        am.insert(db.as_ref()).await?;

        Ok(self.inner)
    }
}
