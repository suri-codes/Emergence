use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZkError {
    #[error("File error: ")]
    FileError(#[from] io::Error),

    #[error("Parse Failure")]
    ParseError(String),
}
