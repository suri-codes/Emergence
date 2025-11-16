// #![warn(missing_debug_implementations, missing_docs)]
mod db;
mod error;
mod id;
mod kasten;
mod link;
mod tag;
mod workspace;
mod zettel;

pub use db::*;
pub use error::*;
pub use id::*;
pub use kasten::*;
pub use link::*;
pub use tag::*;
pub use workspace::*;
pub use zettel::*;

pub type ZkResult<T> = Result<T, ZkError>;
