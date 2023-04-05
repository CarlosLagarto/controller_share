use serde::{Deserialize, Serialize};

use crate::services::electronics::{device_gen::*, scene::*};
use crate::{lib_serde::*, string_concat::*};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DevicesAndScenesData {
    pub devices: Vec<DeviceGen>,
    pub scenes: Vec<Scene>,
}

impl DevicesAndScenesData {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            scenes: Vec::new(),
        }
    }
}

impl Json for DevicesAndScenesData {
    #[inline]
    #[rustfmt::skip]
    fn json(&self) -> JsonResult<String> { data_to_str(&self) }
}
