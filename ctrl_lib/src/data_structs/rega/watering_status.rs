use num_enum::UnsafeFromPrimitive;
use serde::{Deserialize, Serialize};

///
/// Waiting = 0, <br>
/// Running = 1,<br>
/// Suspended = 2,<br>
/// NotExecuted = 3,<br>
/// Terminated = 4,<br>
/// Error = 5,<br>
/// SuspendedTimeout = 6,
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum WateringStatus {
    Waiting = 0,
    Running = 1,
    Suspended = 2,
    NotExecuted = 3,
    Terminated = 4,
    Error = 5,
    SuspendedTimeout = 6,
}

#[allow(clippy::derivable_impls)]
impl Default for WateringStatus {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self { WateringStatus::Waiting }
}
