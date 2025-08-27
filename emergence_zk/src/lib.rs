mod zettel;
pub use zettel::*;

mod error;
mod frontmatter;
mod tag;

pub use error::*;
pub use frontmatter::*;
pub use tag::*;

pub type ZkResult<T> = Result<T, ZkError>;
