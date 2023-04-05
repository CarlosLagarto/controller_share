use serde::{self, Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[allow(clippy::derive_partial_eq_without_eq)]
///Dimension = 1
#[derive(Copy, Display, Clone, Debug, Default, PartialEq, EnumString, Deserialize, Serialize)]
#[repr(u8)]
pub enum Mode {
    Standard = 0,
    Wizard = 1,
    #[default]
    Manual = 2,
}

#[rustfmt::skip]
impl Mode {
    #[inline]
    pub fn is_manual(&self) -> bool { *self == Mode::Manual }

    #[inline]
    pub fn is_standard(&self) -> bool { *self == Mode::Standard }

    #[inline]
    pub fn is_wizard(&self) -> bool { *self == Mode::Wizard }
}
