use std::{env::current_dir, path::PathBuf};

use emergence_zk::{Kasten, ZettelBuilder, ZettelId, ZkGraph, ZkResult, entities};
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
    pub async fn create(self) -> ZkResult<PathBuf> {
        let rand = ZettelId::default();

        let mut pwd = current_dir()?;
        pwd.push(rand.to_string());

        let x = Kasten::new(pwd).await?;

        // created zettels
        for _ in 0..self.num_nodes {
            let z = ZettelBuilder::new(x.root())
                .with_name("test")
                .build(&x.db)
                .await?;
        }

        let ids = entities::zettel::Entity::find()
            .all(x.db.as_ref())
            .await?
            .iter()
            .map(|m| ZettelId::from(m.nanoid.as_str()))
            .collect::<Vec<_>>();

        let mut remaining_edges = self.num_edges;

        let mut rng = rand::rng();

        while remaining_edges > 0 {
            use rand::prelude::*;

            let src = ids.choose(&mut rng).expect("id's are empty?!");
            let dst = ids.choose(&mut rng).expect("id's are empty?!");

            // now we need to make a link between them
            //
            remaining_edges -= 1;
        }

        //

        //

        // i guess first we can create the actual files
        //
        //

        // and then we can create the edges?
        todo!()
    }
}
