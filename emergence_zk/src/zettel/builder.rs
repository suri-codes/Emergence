use std::fs::OpenOptions;
use std::io::Write;

use chrono::Local;
use sea_orm::ActiveModelTrait as _;

use crate::{FrontMatter, Tag, Workspace, Zettel, ZettelId, ZkResult, entities};

pub struct ZettelBuilder<'a> {
    ws: &'a Workspace,
    inner: Zettel,
}

impl<'a> ZettelBuilder<'a> {
    pub fn new(ws: &'a Workspace) -> Self {
        let id = ZettelId::default();

        let zettel_path = {
            let mut project_root = ws.root.clone();
            project_root.push([id.as_str(), ".md"].join(""));
            project_root
        };

        let front_matter = FrontMatter::new("", Local::now().naive_local(), Vec::<String>::new());

        ZettelBuilder {
            ws,
            inner: Zettel {
                id,
                path: zettel_path,
                front_matter,
                content: "".to_owned(),
                tags: Vec::new(),
                links: vec![],
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

    pub fn with_title(mut self, name: impl Into<String>) -> Self {
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

    pub async fn build(mut self) -> ZkResult<Zettel> {
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

        let am = entities::zettel::ActiveModel {
            nanoid: sea_orm::ActiveValue::Set(self.inner.id.to_string()),
            title: sea_orm::ActiveValue::Set(self.inner.front_matter.title.clone()),
            ..Default::default()
        };

        am.insert(self.ws.db.as_ref()).await?;

        Ok(self.inner)
    }
}
