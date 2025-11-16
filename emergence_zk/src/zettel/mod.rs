use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, query::*,
};
use serde::{Deserialize, Serialize};

use crate::{EmergenceDb, Tag, Workspace, ZettelId, ZkError, ZkResult, entities};

use crate::entities::{prelude::*, tag, zettel_tag};

mod frontmatter;
pub use frontmatter::*;

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

    /// uses the id and root to parse out of the root directory
    pub async fn from_id(id: &ZettelId, ws: &Workspace) -> ZkResult<Self> {
        let mut path = ws.root.clone();
        path.push(id.as_str());

        Self::from_path(path, ws).await
    }

    pub async fn from_path(path: impl Into<PathBuf>, ws: &Workspace) -> ZkResult<Self> {
        let path: PathBuf = path.into();

        let id = ZettelId::try_from(path.as_path())?;

        let (front_matter, content) = FrontMatter::extract_from_file(&path)?;

        let mut zettel_tag_strings = front_matter.tag_strings.clone();

        zettel_tag_strings.sort();

        let mut zettel_tags = vec![];

        for tag in ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nanoid(id.as_str())
            .one(ws.db.as_ref())
            .await?
            .expect("zettel missing from db!")
            .tags
            .into_iter()
        {
            if let Ok(idx) = zettel_tag_strings.binary_search(&tag.name) {
                // we remove tags we have already processed
                zettel_tag_strings.remove(idx);
                zettel_tags.push(Tag::from(tag))
            } else {
                // the db says the file has tag `x`, but that tag is missing from the
                // front matter, we can assume its gone
                //
                //
                // so i have a tag id, i need to find the zetteltag
                // link between this zettel and the tag id, and then
                // delete it
                let x = ZettelTag::find()
                    .filter(zettel_tag::Column::ZettelNanoId.eq(id.as_str()))
                    .filter(zettel_tag::Column::TagNanoId.eq(tag.id))
                    .one(ws.db.as_ref())
                    .await?
                    .expect("this link must exist");

                x.into_active_model().delete(ws.db.as_ref()).await?;
            }
        }

        // now any tags that are left inside zettel_tag_strings,
        // we have to put them inside the db
        //
        for new_tag in zettel_tag_strings {
            let am = tag::ActiveModel {
                name: sea_orm::ActiveValue::Set(new_tag),
                color: sea_orm::ActiveValue::Set("random".to_owned()),
                ..Default::default()
            };

            let x = am.insert(ws.db.as_ref()).await?;
            zettel_tags.push(Tag::from(x));
        }

        Ok(Zettel {
            path,
            id,
            front_matter,
            tags: zettel_tags,
            content,
        })
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
