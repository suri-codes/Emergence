use std::fmt::Display;

use crate::{ZkError, ZkResult};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    name: String,
    //TODO: make this actually something
    color: String,
}

impl Tag {
    pub fn new(name: &str, color: &str) -> ZkResult<Self> {
        let name = name.to_lowercase();

        if !name.is_ascii() {
            return Err(ZkError::ParseError("Name isn't valid ascii!"));
        }

        //TODO: color validation or something

        // we can do some parse validation here
        Ok(Self {
            name: name.to_owned(),
            color: color.to_owned(),
        })
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)
    }
}
