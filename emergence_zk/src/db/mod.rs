use std::{collections::HashMap, fs, path::PathBuf};

use petgraph::prelude::StableUnGraph;
use pulldown_cmark::{Event, Parser, Tag};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sea_orm::{Database, DatabaseConnection};
use tracing::error;

use crate::{FrontMatter, Link, Zettel, ZettelId, ZkGraph, ZkResult};

pub mod entities;
pub use sea_orm::entity;

use migration::{Migrator, MigratorTrait};

#[derive(Debug, Clone)]
pub struct EmergenceDb {
    inner: DatabaseConnection,
    root: PathBuf,
}

impl AsRef<DatabaseConnection> for EmergenceDb {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.inner
    }
}

impl EmergenceDb {
    pub async fn connect(root: impl Into<PathBuf>) -> ZkResult<Self> {
        let root_folder = root.into();
        let path = format!(
            "{}/.emergence/emergence.sqlite",
            root_folder.canonicalize()?.as_path().to_string_lossy()
        );

        let db: DatabaseConnection =
            Database::connect(format!("sqlite://{}?mode=rwc", path)).await?;

        // apply all migrations
        Migrator::up(&db, None).await?;
        // synchronizes database schema with entity definitions

        Ok(Self {
            inner: db,
            root: root_folder,
        })
    }

    /// The full sync functin makes it so that all the information in the database
    /// is correct up until the point in time that this function is called
    pub async fn full_sync(&self) -> ZkResult<()> {
        let valid_parsed_files: Vec<_> = fs::read_dir(&self.root)?
            .par_bridge()
            .flatten()
            .filter_map(|entry| match entry.file_type() {
                Ok(ft)
                    if ft.is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "md")
                            .unwrap_or(false) =>
                {
                    // we want to have a type of
                    let f_str = fs::read_to_string(entry.path())
                        .inspect_err(|e| error!("error reading from file {entry:?}: {e:?}"))
                        .ok()?;

                    let (front_matter, content) = FrontMatter::extract_from_str(&f_str)
                        .inspect_err(|e| error!("Error parsing frontmatter for {entry:?}: {e:?}"))
                        .ok()?;

                    let zettel_id: ZettelId = entry
                        .path()
                        .try_into()
                        .inspect_err(|e| {
                            error!("Error parsing ZettelId from {:?}: {e:?}", entry.path())
                        })
                        .ok()?;

                    //TODO: update the metadata
                    // i guess we can update the front_matter right here

                    Some((front_matter, content, entry.path(), zettel_id))
                }
                _ => None,
            })
            .collect();

        let mut valid_zettels = Vec::new();

        let mut path_to_zid = HashMap::new();

        // here we make all the zettels
        for (front_matter, content, path, id) in valid_parsed_files {
            // here we just have to get the tags
            let zettel = Zettel::new(id, path, front_matter, Vec::new(), content);

            let _ = path_to_zid.insert(zettel.path.canonicalize()?, zettel.id.clone());

            valid_zettels.push(zettel)
        }

        // now we can see if the zettels link to eachother
        let mut graph: ZkGraph =
            StableUnGraph::with_capacity(valid_zettels.len(), valid_zettels.len() * 3);

        let mut zid_to_nodeidx = HashMap::new();

        for zettel in valid_zettels.clone() {
            let id = zettel.id.clone();
            let nid = graph.add_node(zettel);
            zid_to_nodeidx.insert(id, nid);
        }

        for zettel in valid_zettels {
            let parsed = Parser::new(&zettel.content);

            for event in parsed {
                if let Event::Start(Tag::Link { dest_url, .. }) = event {
                    println!("Found dest_url: {dest_url:#?}");
                    let dest_path = {
                        let mut tmp_root = self.root.clone();
                        tmp_root.push(dest_url.into_string());
                        tmp_root
                    };
                    let canon_url = match dest_path.canonicalize() {
                        Ok(canon_url) => {
                            println!("Found canon url: {canon_url:#?}");

                            canon_url
                        }
                        Err(_) => {
                            continue;
                        }
                    };

                    if let Some(dest_zid) = path_to_zid.get(&canon_url) {
                        let link = Link::new(&zettel.id, dest_zid);

                        let src = *zid_to_nodeidx.get(&zettel.id).expect("this must exist");
                        let dest = *zid_to_nodeidx.get(dest_zid).expect("this must exist");

                        graph.add_edge(src, dest, link);
                    }
                }
            }
        }

        println!("{graph:#?}");

        Ok(())
    }
}
