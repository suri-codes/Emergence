use std::{env::current_dir, path::PathBuf};

use emergence_zk::{Kasten, Zettel, ZettelBuilder, ZettelId, ZkGraph, ZkResult, entities};
use sea_orm::EntityTrait;

#[expect(unused)]
pub struct ZKreator {
    num_nodes: u32,
    num_edges: u32,

    graph: ZkGraph,
}

impl ZKreator {
    pub fn new(num_nodes: u32, num_edges: u32) -> Self {
        let graph = ZkGraph::with_capacity(num_nodes as usize, num_edges as usize);
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
            let z = ZettelBuilder::new(x.root())
                .with_title("test")
                .build(&x.db)
                .await?;

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
