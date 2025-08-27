mod zettel;
pub use zettel::*;

mod error;
mod frontmatter;
mod kasten;
mod tag;

pub use error::*;
pub use frontmatter::*;
pub use kasten::*;
pub use tag::*;

pub type ZkResult<T> = Result<T, ZkError>;
