use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::RwLock,
};

use log::error;
use petgraph::prelude::{StableGraph, StableUnGraph};
use pulldown_cmark::{Event, Parser, Tag as MkTag};

use crate::{Link, Zettel, ZkError, ZkResult};

pub type ZkGraph = StableUnGraph<Zettel, Link>;

#[derive(Debug)]
pub struct Kasten {
    pub graph: ZkGraph,
    _root: PathBuf,
}

const GRAPH_MAX_NODES: usize = 1024;
const GRAPH_MAX_EDGES: usize = GRAPH_MAX_NODES * 3;

impl Kasten {
    /// Creates a new kasten at the provided `dest`
    ///
    /// # Errors
    /// This function can error if any file-system operation fails.  
    pub fn new(dest: impl Into<PathBuf>) -> ZkResult<Self> {
        let dest: PathBuf = dest.into();

        fs::create_dir_all(&dest)?;

        let mut our_folder = dest.clone();
        our_folder.push(".emergence");

        fs::create_dir_all(our_folder)?;

        let graph: ZkGraph = StableUnGraph::with_capacity(GRAPH_MAX_NODES, GRAPH_MAX_EDGES);
        // okay now we have a new thingy
        let me = Self { graph, _root: dest };

        Ok(me)
    }

    /// Parses a Kasten from the specified `root`.
    /// NOTE: If any `Zettel` is unable to be parsed, it will be skipped instead of erroring out.
    ///
    /// # Errors
    /// This function can error if any file-system operation fails.  
    pub fn parse(root: impl Into<PathBuf>) -> ZkResult<Self> {
        root.into().try_into()
    }

    /// WARN: Blocking
    pub fn watch(&self) {
        todo!()
    }
}

impl TryFrom<PathBuf> for Kasten {
    type Error = ZkError;

    //TODO: Parallelize the shit out of this dawg
    fn try_from(root: PathBuf) -> Result<Self, Self::Error> {
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
                let Ok(zettel) = Zettel::try_from(entry.path().as_path())
                    .inspect_err(|e| error!("Error parsing Zettel: {e:?}"))
                else {
                    // skip file if we arent able to parse it
                    continue;
                };

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
                    println!("Found dest_url: {dest_url:#?}");
                    let dest_path = {
                        let mut tmp_root = root.clone();
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

        let kasten = Kasten { graph, _root: root };

        Ok(kasten)
    }
}
