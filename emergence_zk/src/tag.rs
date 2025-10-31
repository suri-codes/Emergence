use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{ZkError, ZkResult};

//TODO: think about how we want to deal with tags

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tag {
    name: String,
    //TODO: make this actually something
    color: String,
}

impl Tag {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> ZkResult<Self> {
        let name = name.into();
        let color = color.into();
        let name = name.to_lowercase();

        if !name.is_ascii() {
            return Err(ZkError::ParseError("Name isn't valid ascii!".to_owned()));
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
        write!(f, "${}{{#{}}}", self.name, self.color)
    }
}
