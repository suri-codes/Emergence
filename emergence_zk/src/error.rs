use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZkError {
    #[error("File error: ")]
    FileError(#[from] io::Error),

    #[error("Parse Error: ")]
    ParseError(String),

    #[error("Database Error: ")]
    DbError(#[from] sea_orm::DbErr),

    #[error("FS Watcher Error")]
    NotifyError(#[from] notify::Error),

    
}
