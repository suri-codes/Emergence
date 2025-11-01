use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::Mutex,
};

use log::error;
use petgraph::prelude::StableUnGraph;
use pulldown_cmark::{Event, Parser, Tag as MkTag};
use rayon::prelude::*;

use crate::{FrontMatter, Link, Metadata, Zettel, ZettelId, ZkError, ZkResult};

pub type ZkGraph = StableUnGraph<Zettel, Link>;

#[derive(Debug)]
pub struct Kasten {
    pub graph: ZkGraph,
    _root: PathBuf,
}

/// maximum number of nodes in our graph, setting at this arbitrary number because im not sure
/// if the graph type has the capability to scale with adding more nodes
const GRAPH_MAX_NODES: usize = 128;
/// Arbitrarily chosen maximum number of edges
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
        // get metadata
        let mut _metadata = Metadata::parse(root.clone())
            .map_err(|e| ZkError::ParseError(format!("Failed to parse metadata: {e:?}")))?;

        let valid_parsed_files: Vec<_> = fs::read_dir(&root)?
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
