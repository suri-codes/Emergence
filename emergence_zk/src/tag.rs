use std::{collections::HashMap, fmt::Display, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{ZkError, ZkResult, entities::tag};

//TODO: think about how we want to deal with tags

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    //TODO: make this actually something
    pub color: String,
}

pub type TagMap = HashMap<String, Tag>;

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

impl From<tag::ModelEx> for Tag {
    fn from(value: tag::ModelEx) -> Self {
        Tag {
            name: value.name,
            color: value.color,
        }
    }
}
impl From<tag::Model> for Tag {
    fn from(value: tag::Model) -> Self {
        Tag {
            name: value.name,
            color: value.color,
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
