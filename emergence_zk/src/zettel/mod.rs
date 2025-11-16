use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{Tag, Workspace, ZettelId, ZkResult};

use crate::entities::{prelude::*, tag, zettel, zettel_tag};

mod frontmatter;
pub use frontmatter::*;
mod builder;
pub use builder::*;

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

        let db_zettel = ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nanoid(id.as_str())
            .one(ws.db.as_ref())
            .await?
            .unwrap_or_else(|| panic!("zettel missing from db! :{id:?}"));

        for db_tag in db_zettel.tags.into_iter() {
            if let Ok(idx) = zettel_tag_strings.binary_search(&db_tag.name) {
                // we remove tags we have already processed
                zettel_tag_strings.remove(idx);
                zettel_tags.push(Tag::from(db_tag))
            } else {
                // the db says the file has tag `x`, but that tag is missing from the
                // front matter, we can assume its gone, lets delete that link
                let x = ZettelTag::find()
                    .filter(zettel_tag::Column::ZettelNanoId.eq(id.as_str()))
                    .filter(zettel_tag::Column::TagNanoId.eq(db_tag.nanoid))
                    .one(ws.db.as_ref())
                    .await?
                    .expect("this link must exist");

                x.into_active_model().delete(ws.db.as_ref()).await?;
            }
        }

        // now any tags that are left inside zettel_tag_strings,
        // we have to put them inside the db
        for new_tag in zettel_tag_strings {
            let am = tag::ActiveModel {
                nanoid: sea_orm::ActiveValue::Set(ZettelId::default().to_string()),
                name: sea_orm::ActiveValue::Set(new_tag),
                color: sea_orm::ActiveValue::Set("random".to_owned()),

                ..Default::default()
            };

            let x = am.insert(ws.db.as_ref()).await?;

            let am = zettel_tag::ActiveModel {
                zettel_nano_id: sea_orm::ActiveValue::Set(id.to_string()),
                tag_nano_id: sea_orm::ActiveValue::Set(x.nanoid.clone()),
            };

            let _ = am.insert(ws.db.as_ref()).await?;

            zettel_tags.push(Tag::from(x));
        }

        if front_matter.title != db_zettel.title {
            let am = zettel::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(db_zettel.id),
                title: sea_orm::ActiveValue::Set(front_matter.title.clone()),
                ..Default::default()
            };

            am.update(ws.db.as_ref()).await?;
        }

        Ok(Zettel {
            path,
            id,
            front_matter,
            tags: zettel_tags,
            content,
        })
    }

    /// Writes this Zettel to Disk
    pub fn flush(&self) -> ZkResult<()> {
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(self.path.as_path())?;

        write!(f, "{self}")?;
        Ok(())
    }
}

impl Display for Zettel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.front_matter)?;
        write!(f, "{}", self.content)
    }
}
