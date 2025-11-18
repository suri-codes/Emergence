use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use egui_graphs::Node;
use pulldown_cmark::{Event, Parser, Tag as MkTag};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{Link, Tag, Workspace, ZettelId, ZkResult, entities};

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
    pub links: Vec<Link>,
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
            links: vec![],
            content,
        }
    }

    /// uses the id and root to parse out of the root directory
    pub async fn from_id(id: &ZettelId, ws: &Workspace) -> ZkResult<Self> {
        let mut path = ws.root.clone();
        path.push(id.as_str());

        Self::from_path(path, ws).await
    }

    fn links_from_content(src_id: &ZettelId, content: &str, ws: &Workspace) -> ZkResult<Vec<Link>> {
        let parsed = Parser::new(content);

        let mut links = vec![];

        for event in parsed {
            if let Event::Start(MkTag::Link { dest_url, .. }) = event {
                info!("Found dest_url: {dest_url:#?}");

                let dest_path = {
                    // remove leading "./"
                    let without_prefix = dest_url.strip_prefix("./").unwrap_or(&dest_url);

                    // remove "#" and everything after it
                    let without_anchor = without_prefix.split('#').next().unwrap();

                    // add .md if not present
                    let normalized = if without_anchor.ends_with(".md") {
                        without_anchor.to_string()
                    } else {
                        format!("{}.md", without_anchor)
                    };

                    let mut tmp_root = ws.root.clone();
                    tmp_root.push(normalized);
                    tmp_root
                };
                // simplest way to validate that the path exists
                let canon_url = match dest_path.canonicalize() {
                    Ok(canon_url) => canon_url,
                    Err(_) => {
                        error!("Link not found!: {dest_path:?}");
                        continue;
                    }
                };

                let dst_id = ZettelId::try_from(canon_url)?;

                let link = Link::new(src_id, dst_id);

                links.push(link)
            }
        }

        Ok(links)
    }

    pub async fn from_path(path: impl Into<PathBuf>, ws: &Workspace) -> ZkResult<Self> {
        let path: PathBuf = path.into();

        let id = ZettelId::try_from(path.as_path())?;

        let (front_matter, content) = FrontMatter::extract_from_file(&path)?;

        let mut zettel_tag_strings = front_matter.tag_strings.clone();

        zettel_tag_strings.sort();

        let mut zettel_tags = vec![];

        // this should probably work like it
        let db_zettel = if let Some(z) = ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nanoid(id.as_str())
            .one(ws.db.as_ref())
            .await?
        {
            z
        } else {
            // if zettel is missing from db, we just add it here
            info!("adding zettel to db");
            let am = entities::zettel::ActiveModel {
                nanoid: sea_orm::ActiveValue::Set(id.to_string()),
                title: sea_orm::ActiveValue::Set(front_matter.title.clone()),
                ..Default::default()
            };

            am.insert(ws.db.as_ref()).await?;

            ZettelEntity::load()
                .with(TagEntity)
                .filter_by_nanoid(id.as_str())
                .one(ws.db.as_ref())
                .await?
                .expect("we just inserted the zettel")
        };

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

        // now we can do links
        //
        let links = Self::links_from_content(&id, &content, ws)?;

        Ok(Zettel {
            path,
            id,
            front_matter,
            tags: zettel_tags,
            content,
            links,
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

    pub fn apply_node_transform(&self, node: &mut Node<Zettel, Link>) {
        node.set_label(self.front_matter.title.to_owned());
        let disp = node.display_mut();
        disp.radius = 100.0;
    }
}

impl Display for Zettel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.front_matter)?;
        write!(f, "{}", self.content)
    }
}
