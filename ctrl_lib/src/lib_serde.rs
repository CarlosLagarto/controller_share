use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use thiserror::*;

pub trait Json {
    fn json(&self) -> JsonResult<String>;
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Technical: Issue converting event to a string: {0}")]
    SerializationIssue(String),
    #[error("Technical: Issue converting string: {0}\n Error: {1}")]
    DeserializationIssue(String, String),
}

pub type JsonResult<T> = std::result::Result<T, ConversionError>;

#[inline]
pub fn data_from_str<'a, T>(s: &'a str) -> JsonResult<T>
where
    T: Deserialize<'a>,
{
    from_str(s).map_err(|err| ConversionError::DeserializationIssue(s.to_string(), err.to_string()))
}

#[inline]
pub fn data_to_str<T>(value: &T) -> JsonResult<String>
where
    T: Serialize,
{
    to_string(&value).map_err(|err| ConversionError::SerializationIssue(err.to_string()))
}
