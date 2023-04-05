use std::result::Result;

use thiserror::*;

pub type DBResult<T> = Result<T, DBError>;
pub type SimpleResult = Result<(), DBError>;

#[allow(clippy::large_enum_variant)] //is não vai acontecer.  Está aqui porque...nao sei...
#[derive(Error, Debug)]
pub enum DBError {
    #[error("Error from database engine")]
    SqlError(#[from] rusqlite::Error),
    #[error("Trying to execute a sql instruction in a transaction with no transaction defined.")]
    TransactionWithNoBegin,
    #[error("No database available.")]
    NoDatabaseAvailable,
    #[error("No connection to op database: {0}")]
    OpDBError(String),
    #[error("Error accessing op database file")]
    IOError(#[from] std::io::Error),
}
