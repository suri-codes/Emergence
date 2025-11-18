use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use notify::{RecursiveMode, Watcher};
use petgraph::{Directed, prelude::NodeIndex, prelude::StableGraph};
use rayon::prelude::*;
use tokio::time::Instant;
use tracing::{error, info};

use crate::{Link, Workspace, Zettel, ZettelId, ZkResult};
use egui_graphs::Graph;

pub type ZkGraph = Graph<Zettel, Link, Directed>;

#[derive(Debug, Clone)]
pub struct Kasten {
    pub id: ZettelId,
    pub graph: ZkGraph,
    pub ws: Workspace,
    pub zid_to_gid: HashMap<ZettelId, NodeIndex>,
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

        let graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(
            GRAPH_MAX_NODES,
            GRAPH_MAX_EDGES,
        ));

        let ws = Workspace::new(&dest).await?;

        let id = ZettelId::default();
        // okay now we have a new thingy
        let me = Self {
            id,
            graph,

            ws,
            zid_to_gid: HashMap::new(),
        };

        Ok(me)
    }

    /// Parses a Kasten from the specified `root`.
    /// NOTE: If any `Zettel` is unable to be parsed, it will be skipped instead of erroring out.
    ///
    /// # Errors
    /// This function can error if any file-system operation fails.  
    pub async fn parse(root: impl Into<PathBuf>) -> ZkResult<Self> {
        let start = Instant::now();
        let root = root.into();

        let ws = Workspace::new(&root).await?;

        let paths = fs::read_dir(&root)?
            .par_bridge()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "md")
                        .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        // spawn all the zettel tasks
        let zettel_tasks = paths
            .into_iter()
            .map(|path| {
                let ws = ws.clone(); // Clone Arc or whatever ws is
                tokio::spawn(async move { Zettel::from_path(path, &ws).await })
            })
            .collect::<Vec<_>>();

        // await all of them
        let zettels = futures::future::join_all(zettel_tasks)
            .await
            .into_iter()
            .filter_map(
                |result| result.ok()?.ok(), // .map(|z| Arc::new(z) )
            )
            .collect::<Vec<Zettel>>();
        let mut graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(
            zettels.len(),
            zettels.len() * 3,
        ));

        // now we have to update the graph

        let mut zid_to_gid = HashMap::new();
        for zettel in &zettels {
            let id = graph.add_node_custom(zettel.clone(), |node| {
                zettel.apply_node_transform(node);
            });
            zid_to_gid.insert(zettel.id.clone(), id);
        }

        for zettel in &zettels {
            let src = zid_to_gid.get(&zettel.id).expect("must exist");
            for link in &zettel.links {
                let dst = zid_to_gid.get(&link.dest).expect("must exist");
                graph.add_edge(*src, *dst, link.clone());
            }
        }

        info!("graph: {graph:#?}");

        let kasten = Kasten {
            id: ZettelId::default(),
            graph,
            ws,
            zid_to_gid,
        };

        let end = start.elapsed();

        println!("time taken to parse workspace: {end:#?}");

        Ok(kasten)
    }

    pub async fn watch(&mut self) -> ZkResult<()> {
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        // Use recommended_watcher() to automatically select the best implementation
        // for your platform. The `EventHandler` passed to this constructor can be a
        // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
        // another type the trait is implemented for.
        let mut watcher = notify::recommended_watcher(tx)?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(Path::new(&self.ws.root), RecursiveMode::Recursive)?;
        // Block forever, printing out events as they come in

        while let Ok(res) = rx.recv() {
            match res {
                Ok(event) => {
                    info!("event: {:#?}", event);
                    if let notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) =
                        event.kind
                    {
                        for path in event.paths {
                            info!("we are goin through shit");
                            let Ok(z) = Zettel::from_path(&path, &self.ws).await.inspect_err(|e| {
                                error!("Unable to parse zettel from path: {path:#?}, error: {e:#?}")
                            }) else {
                                continue;
                            };

                            info!("zettel: {z:#?}");
                            let mut graph = &mut self.graph;
                            // actually this has the very real possibility of changing :grin:
                            let gid = {
                                match self.zid_to_gid.get(&z.id) {
                                    Some(gid) => *gid,
                                    None => {
                                        // this zettel was created while we have watch open, lets just add
                                        // it to thegraph and the hashmap
                                        let gid = graph.add_node_custom(z.clone(), |node| {
                                            z.apply_node_transform(node)
                                        });

                                        self.zid_to_gid.insert(z.id.clone(), gid);
                                        gid
                                    }
                                }
                            };

                            let x = graph.g_mut().node_weight_mut(gid).expect("must exist");
                            (*x.payload_mut()) = z.clone();
                            z.apply_node_transform(x);

                            let curr_edgs = graph
                                .g()
                                .edges(gid)
                                .map(|e| e.weight().id())
                                .collect::<Vec<_>>();

                            for edge in curr_edgs {
                                let _ = graph.remove_edge(edge);
                            }

                            for link in z.links {
                                let dest = self.zid_to_gid.get(&link.dest).expect("must exist");
                                graph.add_edge(gid, *dest, link);
                            }

                            info!("graph: {graph:#?}");
                            drop(graph)
                        }
                    }
                }
                Err(e) => error!("watch error: {:#?}", e),
            }
        }

        Ok(())
    }
}
