use num_enum::UnsafeFromPrimitive;
use serde::{self, Deserialize, Serialize};
use thiserror::*;

#[allow(clippy::derive_partial_eq_without_eq)]

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, UnsafeFromPrimitive)]
/// Never <br>
/// SpecificWeekday<br>
/// Every<br>
#[repr(u8)]
pub enum ScheduleRepeat {
    Never = 0,
    SpecificWeekday = 1,
    Every = 2,
}

#[allow(clippy::derivable_impls)]
impl Default for ScheduleRepeat {
    fn default() -> Self {
        ScheduleRepeat::Never
    }
}

impl ScheduleRepeat {
    #[inline]
    pub fn description(&self) -> &str {
        match *self {
            ScheduleRepeat::Never => "Never",
            ScheduleRepeat::SpecificWeekday => "SpecificWeekday",
            ScheduleRepeat::Every => "Every",
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, UnsafeFromPrimitive)]
#[repr(u8)]
/// Never<br>
/// Retries<br>
/// Date<br>
pub enum ScheduleStop {
    Never = 0,
    Retries = 1,
    Date = 2,
}

#[allow(clippy::derivable_impls)]
impl Default for ScheduleStop {
    #[rustfmt::skip]
    #[inline]
    fn default() -> Self { ScheduleStop::Never }
}

impl ScheduleStop {
    #[inline]
    pub fn description(&self) -> &str {
        match *self {
            ScheduleStop::Never => "Never",
            ScheduleStop::Retries => "Retries",
            ScheduleStop::Date => "Date",
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, UnsafeFromPrimitive)]
#[repr(u8)]
/// Seconds<br>
/// Minutes<br>
/// Hours<br> 
/// Days<br> 
/// Weeks<br>
pub enum ScheduleRepeatUnit {
    Seconds = 0,
    Minutes = 1,
    Hours = 2,
    Days = 3,
    Weeks = 4,
}

#[allow(clippy::derivable_impls)]
impl Default for ScheduleRepeatUnit {
    #[rustfmt::skip]
    #[inline]
    fn default() -> Self { ScheduleRepeatUnit::Days }
}

/// InvalidStopDefinition<br>
/// RepeatKindWithNoQty <br>
/// InvalidRepeatKind <br>
#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error("Invalid Stop Condition; {stop_condition:?}")]
    InvalidStopDefinition { stop_condition: String },
    #[error("Repeat Kind: {repeat_kind:?} with zero retries ")]
    RepeatKindWithNoQty { repeat_kind: String },
    #[error("Invalid Repeat Kind: {repeat_kind:?}")]
    InvalidRepeatKind { repeat_kind: String },
}
