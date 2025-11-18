use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use notify::{
    Config, EventKind, RecommendedWatcher,
    event::{ModifyKind, RemoveKind},
};
use notify::{RecursiveMode, Watcher};
use petgraph::{Directed, prelude::NodeIndex, prelude::StableGraph};
use rayon::prelude::*;
use tokio::{sync::mpsc::channel, time::Instant};
use tracing::{error, info, warn};

use crate::{Link, Workspace, Zettel, ZettelId, ZkResult};
use egui_graphs::Graph;

pub type ZkGraph = Graph<Zettel, Link, Directed>;

#[derive(Debug, Clone)]
pub struct Kasten {
    pub id: ZettelId,
    pub name: String,
    pub graph: ZkGraph,
    pub ws: Workspace,
    pub zid_to_gid: HashMap<ZettelId, NodeIndex>,
}

pub type KastenHandle = Arc<Mutex<Kasten>>;

/// maximum number of nodes in our graph, setting at this arbitrary number because im not sure
/// if the graph type has the capability to scale with adding more nodes
const GRAPH_MAX_NODES: usize = 128;
/// Arbitrarily chosen maximum number of edges
const GRAPH_MAX_EDGES: usize = GRAPH_MAX_NODES * 3;

impl Kasten {
    fn name_from_path_buf(path: PathBuf) -> String {
        path.file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or("ZettleKasten".to_owned())
    }

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
            name: Self::name_from_path_buf(dest),
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

                let x = rand::random_range(0.0..=100.0);
                let y = rand::random_range(0.0..=100.0);
                node.set_location(emath::Pos2 { x, y });
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
            name: Self::name_from_path_buf(root),
            graph,
            ws,
            zid_to_gid,
        };

        let end = start.elapsed();

        println!("time taken to parse workspace: {end:#?}");

        Ok(kasten)
    }

    /// NOTE: This function will block forever
    /// Will watch the underlying folder and apply any file changes to the `ZKGraph` of this `Kasten`
    pub async fn watch(k_handle: KastenHandle) -> ZkResult<()> {
        info!(
            "watching kasten: {:#?}",
            k_handle.lock().expect("should never be poisoned").id
        );

        let ws = k_handle.lock().expect("lol").ws.clone();
        let (tx, mut rx) = channel(1);

        let mut watcher = RecommendedWatcher::new(
            move |res| tx.blocking_send(res).expect("failed to send event"),
            Config::default(),
        )?;

        watcher
            .watch(Path::new(&ws.root), RecursiveMode::Recursive)
            .expect("unable to start watching");

        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => {
                    info!("fs event: {:#?}", event);

                    match event.kind {
                        // a file was deleted
                        EventKind::Remove(RemoveKind::Any | RemoveKind::File)
                        | EventKind::Modify(ModifyKind::Name(_)) => {
                            // this is the path that was removed
                            for path in event.paths {
                                let Ok(id) = ZettelId::try_from(path) else {
                                    continue;
                                };

                                info!("deleting zettel: {id:#?}");

                                let mut kasten_guard =
                                    k_handle.lock().expect("lock must not be poisoned");

                                let Some(g_id) = kasten_guard.zid_to_gid.get(&id).copied() else {
                                    warn!(
                                        "the id we were trying to delete didnt exist inside zid_to_gid, skipping"
                                    );

                                    continue;
                                };

                                // remove from graph
                                let _ = kasten_guard.graph.remove_node(g_id);
                                kasten_guard.zid_to_gid.remove(&id);
                            }
                        }
                        EventKind::Modify(ModifyKind::Data(_)) => {
                            for path in event.paths {
                                let Ok(z) = Zettel::from_path(&path, &ws).await.inspect_err(|e| {
                                    error!(
                                        "Unable to parse zettel from path: {path:#?}, error: {e:#?}"
                                    )
                                }) else {
                                    continue;
                                };

                                info!("Processing content change in zettel: {z:#?}");

                                let mut kasten_guard =
                                    k_handle.lock().expect("lock must not be poisoned");

                                let gid = {
                                    match kasten_guard.zid_to_gid.get(&z.id) {
                                        Some(gid) => *gid,
                                        None => {
                                            // this zettel was created while we have watch open, lets just add
                                            // it to kasten_guard.thegraph and the hashmap
                                            let gid = kasten_guard
                                                .graph
                                                .add_node_custom(z.clone(), |node| {
                                                    z.apply_node_transform(node)
                                                });

                                            kasten_guard.zid_to_gid.insert(z.id.clone(), gid);
                                            gid
                                        }
                                    }
                                };

                                let curr_edgs = kasten_guard
                                    .graph
                                    .g()
                                    .edges(gid)
                                    .map(|e| e.weight().id())
                                    .collect::<Vec<_>>();

                                for edge in curr_edgs {
                                    let _ = kasten_guard.graph.remove_edge(edge);
                                }

                                for link in z.links {
                                    let dest = *kasten_guard
                                        .zid_to_gid
                                        .get(&link.dest)
                                        .expect("must exist");
                                    kasten_guard.graph.add_edge(gid, dest, link);
                                }
                            }
                        }

                        _ => {}
                    }
                }
                Err(e) => error!("watch error: {:#?}", e),
            }
        }

        Ok(())
    }
}
