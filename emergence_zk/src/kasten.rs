use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::RwLock,
};

use petgraph::{adj::NodeIndex, prelude::StableUnGraph};
use pulldown_cmark::{Event, Parser, Tag as MkTag};

use crate::{Link, Zettel, ZkError, ZkResult};

// pub type Link = (ZettelId, ZettelId);

pub type ZkGraph = StableUnGraph<Zettel, Link>;

struct Kasten {
    pub graph: RwLock<ZkGraph>,
    root: PathBuf,
}

impl Kasten {
    //TODO: Parallelize the shit out of this dawg
    fn generate(root: PathBuf) -> ZkResult<Self> {
        let mut valid_zettels = Vec::new();

        let mut path_to_zid = HashMap::new();

        for entry in fs::read_dir(root.clone())? {
            let entry = entry?;

            if let Some(end_bit) = entry
                .file_name()
                .into_string()
                .map_err(|os_str| {
                    ZkError::ParseError(format!(
                        "Failed to convert file name: {os_str:?} into a proper string!"
                    ))
                })?
                .split_terminator(".")
                .last()
                && end_bit == "md"
                && entry.file_type()?.is_file()
            {
                let zettel = Zettel::try_from(entry.path().as_path())?;

                let _ = path_to_zid.insert(zettel.path.canonicalize()?, zettel.id.clone());

                valid_zettels.push(zettel)
            }
        }

        // now we can see if the zettels link to eachother
        //
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
                if let Event::Start(MkTag::Link { dest_url, .. }) = event {
                    let canon_url = match PathBuf::from(dest_url.to_string()).canonicalize() {
                        Ok(canon_url) => canon_url,
                        Err(_) => continue,
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

        let kasten = Kasten {
            graph: RwLock::new(graph),
            root,
        };

        Ok(kasten)
    }

    /// WARN: Blocking
    pub fn watch(&self) {
        todo!()
    }
}
