use std::{env::current_dir, path::PathBuf};

use emergence_zk::{Kasten, ZettelBuilder, ZettelId, ZkGraph, ZkResult};
use petgraph::prelude::StableGraph;

#[expect(unused)]
pub struct ZKreator {
    num_nodes: usize,
    num_edges: usize,

    graph: ZkGraph,
}

impl ZKreator {
    pub fn new(num_nodes: usize, num_edges: usize) -> Self {
        let graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(num_nodes, num_edges));
        ZKreator {
            num_nodes,
            num_edges,
            graph,
        }
    }

    /// creates allat
    #[expect(unused)]
    pub async fn create(mut self) -> ZkResult<PathBuf> {
        let root = {
            let rand = ZettelId::default();
            let mut pwd = current_dir()?;
            pwd.push(format!("zkreator_{}", rand));
            pwd
        };

        let x = Kasten::new(&root).await?;

        let ws = &x.ws;

        // created zettels
        let mut zettels = Vec::new();
        for _ in 0..self.num_nodes {
            let z = ZettelBuilder::new(ws).with_title("test").build().await?;

            zettels.push(z.clone());
        }
        let mut remaining_edges = self.num_edges;

        let mut rng = rand::rng();

        while remaining_edges > 0 {
            use rand::prelude::*;

            let dst = zettels
                .choose(&mut rng)
                .expect("id's are empty?!")
                .id
                .clone();

            let src = zettels.choose_mut(&mut rng).expect("id's are empty?!");

            src.content.push_str(format!("[dst]({dst}.md)\n").as_str());
            src.flush();
            remaining_edges -= 1;
        }

        Ok(root)
    }
}
