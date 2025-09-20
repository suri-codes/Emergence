mod error;
mod frontmatter;
mod kasten;
mod tag;
mod zettel;

pub use error::*;
pub use frontmatter::*;
pub use kasten::*;
pub use tag::*;
pub use zettel::*;

pub type ZkResult<T> = Result<T, ZkError>;
