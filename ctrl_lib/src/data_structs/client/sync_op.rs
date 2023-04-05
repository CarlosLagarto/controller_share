use serde::{self, Serialize, Deserialize};

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum SyncOp {
    #[default]
    I,
    D,
    U,
}

impl SyncOp {
    #[inline]
    pub fn to_str(&self) -> String {
        match self {
            SyncOp::I => "I".to_owned(),
            SyncOp::D => "D".to_owned(),
            SyncOp::U => "U".to_owned(),
        }
    }
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str(str: &str) -> SyncOp {
        match str {
            "I" => SyncOp::I,
            "D" => SyncOp::D,
            "U" => SyncOp::U,
            _ => unreachable!(),
        }
    }
}
