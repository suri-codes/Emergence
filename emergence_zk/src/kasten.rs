use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use petgraph::prelude::StableUnGraph;

use crate::{Zettel, ZettelId};

pub type Link = (ZettelId, ZettelId);

pub type ZkGraph = StableUnGraph<Zettel, Link>;

struct Kasten {
    pub graph: RwLock<ZkGraph>,
    root: PathBuf,
}

impl Kasten {
    fn generate(root: PathBuf) -> Self {

        
        todo!()
    }

    /// WARN: Blocking
    pub fn watch(&self) {
        todo!()
    }
}
