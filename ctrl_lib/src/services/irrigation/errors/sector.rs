use thiserror::*;

#[derive(Debug, Error)]
pub enum SecErr {
    #[error("Unknown error closing valve for sector: {sec_name:?}")]
    UnknownErrCloseValve { sec_name: String },
    #[error("Unknown error openning valve for sector: {sec_name:?}")]
    UnknownErrOpenValve { sec_name: String },
}

pub type SectorResult<T> = Result<T, SecErr>;
