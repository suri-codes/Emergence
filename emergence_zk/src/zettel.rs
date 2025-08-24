use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use nanoid::nanoid;

use crate::ZkResult;

pub struct ZettelId(String);

impl ZettelId {
    fn new() -> Self {
        ZettelId(nanoid!())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_owned())
    }
}

pub struct Zettel {
    id: ZettelId,
    path: PathBuf,
}

//TODO: MAKE A ZETTEL BUILDER PATTEN IMPLEMENTATION, THAT WOULD BE SO FUCKING COOL

impl Zettel {
    pub fn new(mut project_root: PathBuf) -> ZkResult<Zettel> {
        let id = ZettelId::new();

        let zettel_path = {
            project_root.push(id.as_str());
            project_root
        };

        // create the new file
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&zettel_path)?;

        Ok(Zettel {
            id,
            path: zettel_path,
        })
    }
}
