mod error;
mod frontmatter;
mod id;
mod kasten;
mod link;
mod tag;
mod zettel;

pub use error::*;
pub use frontmatter::*;
pub use id::*;
pub use kasten::*;
pub use link::*;
pub use tag::*;
pub use zettel::*;

pub type ZkResult<T> = Result<T, ZkError>;
