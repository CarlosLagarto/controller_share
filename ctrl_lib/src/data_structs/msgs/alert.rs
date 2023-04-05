use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{ext_message::*, topic::*};

const FLAG_RAIN_OR_WIND: u8 = 249;
///
/// NoAlert = 0 <br>
/// WIND = 2, <br>
/// RAIN = 4, <br>
/// PresenceDetection = 8, <br>
/// WindowOrDoorOpen = 16, <br>
///  <br>
/// Dimension 1
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, Display, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AlertType {
    NoAlert = 0,
    WIND = 2,
    RAIN = 4,
    PresenceDetection = 8,
    WindowOrDoorOpen = 16,
}

impl AlertType {
    #[inline]
    pub const fn is_rain_or_wind(&self) -> bool {
        *self as u8 & FLAG_RAIN_OR_WIND > 0
    }
}

/// Dimension = 72
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub header: Option<Header>,
    #[serde(rename = "alert_data")]
    pub value: f32,
    #[serde(rename = "alert_type")]
    pub type_: AlertType,
}

impl PartialEq for Alert {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && (self.value - other.value).abs() < f32::EPSILON
    }
}

impl Alert {
    #[inline]
    #[rustfmt::skip]
    pub const fn new(type_: AlertType, value: f32) -> Alert { Alert { header: None, type_, value } }

    #[inline]
    pub fn new_out(data: ExtMsgOut, time: CtrlTime) -> ExtMsgOut {
        ExtMsgOut::new(Topic::STC_SND_ALERT, data, time)
    }
}
