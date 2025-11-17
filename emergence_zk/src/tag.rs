use std::{collections::HashMap, fmt::Display};

use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::{Workspace, ZettelId, ZkError, ZkResult, entities::prelude::*, entities::tag};

//TODO: think about how we want to deal with tags

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    //TODO: make this actually something
    pub color: String,
}

pub type TagMap = HashMap<String, Tag>;

impl Tag {
    pub async fn new(
        name: impl Into<String>,
        color: impl Into<String>,
        ws: &Workspace,
    ) -> ZkResult<Self> {
        let name = name.into();
        let color = color.into();
        let name = name.to_lowercase();

        if !name.is_ascii() {
            return Err(ZkError::ParseError("Name isn't valid ascii!".to_owned()));
        }

        let _ = tag::ActiveModel {
            nanoid: sea_orm::ActiveValue::Set(ZettelId::default().to_string()),
            name: Set(name.to_owned()),
            color: Set(color.to_owned()),
            ..Default::default()
        }
        .save(ws.db.as_ref())
        .await?;

        //TODO: color validation or something

        // we can do some parse validation here
        Ok(Self {
            name: name.to_owned(),
            color: color.to_owned(),
        })
    }

    pub async fn get_or_new(name: impl Into<String>, ws: &Workspace) -> ZkResult<Self> {
        let name = name.into();
        if let Some(existing) = TagEntity::find_by_name(&name).one(ws.db.as_ref()).await? {
            Ok(existing.into())
        } else {
            Self::new(name, "random!", ws).await
        }
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
