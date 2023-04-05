use num_enum::TryFromPrimitive;
use strum_macros::Display;

/// OPEN <br>
/// CLOSE <br>
/// ERROR <br>
#[allow(non_camel_case_types)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Display, Default, Debug, Copy, Clone, PartialEq, TryFromPrimitive)]
#[repr(i8)]
pub enum RelayState {
    Open = 1,   //its watering
    #[default]
    Closed = 0, //its not watering
    Error = -1,
}

#[rustfmt::skip]
impl RelayState {
    #[inline]
    pub fn is_closed(&self) -> bool { *self == RelayState::Closed }

    #[inline]
    pub fn is_error(&self) -> bool { *self == RelayState::Error }

    #[inline]
    pub fn is_open(&self) -> bool { *self == RelayState::Open }

    #[inline]
    pub fn is_error_or_open(&self)->bool{ *self == RelayState::Error || *self == RelayState::Open }

    #[inline]
    pub fn is_error_or_closed(&self)->bool{ *self == RelayState::Error || *self == RelayState::Closed }

}