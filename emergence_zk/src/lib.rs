// #![warn(missing_debug_implementations, missing_docs)]
mod db;
mod error;
mod frontmatter;
mod id;
mod kasten;
mod link;
mod metadata;
mod tag;
mod zettel;

pub use db::*;
pub use error::*;
pub use frontmatter::*;
pub use id::*;
pub use kasten::*;
pub use link::*;
pub use metadata::*;
pub use tag::*;
pub use zettel::*;

pub type ZkResult<T> = Result<T, ZkError>;
