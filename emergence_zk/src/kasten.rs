use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use log::error;
use notify::{RecursiveMode, Watcher};
use petgraph::prelude::StableUnGraph;
use pulldown_cmark::{Event, Parser, Tag as MkTag};
use rayon::prelude::*;

use crate::{EmergenceDb, FrontMatter, Link, Zettel, ZettelId, ZkError, ZkResult};

pub type ZkGraph = StableUnGraph<Zettel, Link>;

#[derive(Debug, Clone)]
pub struct Kasten {
    pub graph: Arc<Mutex<ZkGraph>>,
    pub db: EmergenceDb,
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
    pub async fn new(dest: impl Into<PathBuf>) -> ZkResult<Self> {
        let dest: PathBuf = dest.into();

        fs::create_dir_all(&dest)?;

        let mut our_folder = dest.clone();
        our_folder.push(".emergence");

        fs::create_dir_all(our_folder)?;

        let graph: ZkGraph = StableUnGraph::with_capacity(GRAPH_MAX_NODES, GRAPH_MAX_EDGES);

        let db = EmergenceDb::connect(dest.clone()).await?;

        // okay now we have a new thingy
        let me = Self {
            graph: Arc::new(Mutex::new(graph)),
            _root: dest,
            db,
        };

        Ok(me)
    }

    pub fn root(&self) -> &Path {
        &self._root
    }

    /// Parses a Kasten from the specified `root`.
    /// NOTE: If any `Zettel` is unable to be parsed, it will be skipped instead of erroring out.
    ///
    /// # Errors
    /// This function can error if any file-system operation fails.  
    pub async fn parse(root: impl Into<PathBuf>) -> ZkResult<Self> {
        let root = root.into();

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

        // now we can connect to db

        let db = EmergenceDb::connect(root.clone()).await?;
        let kasten = Kasten {
            graph: Arc::new(Mutex::new(graph)),
            _root: root,
            db,
        };

        Ok(kasten)
    }

    /// WARN: Blocking
    pub fn watch(&self) -> ZkResult<()> {
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        // Use recommended_watcher() to automatically select the best implementation
        // for your platform. The `EventHandler` passed to this constructor can be a
        // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
        // another type the trait is implemented for.
        let mut watcher = notify::recommended_watcher(tx)?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(Path::new(&self._root), RecursiveMode::Recursive)?;
        // Block forever, printing out events as they come in

        loop {
            for res in &rx {
                match res {
                    Ok(event) => println!("event: {:#?}", event),
                    Err(e) => println!("watch error: {:#?}", e),
                }
            }
        }
    }
}
