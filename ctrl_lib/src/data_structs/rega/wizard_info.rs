use crate::app_time::ctrl_time::*;

pub const DAILY_TGT_GRASS_ET: f32 = 3.571_428_5;
pub const GRASS_ROOT_LENGTH: f32 = 150.;

/// Dimension = 16
pub struct WizardInfo {
    pub last_stress_control_time: CtrlTime,
    pub daily_tgt_grass_et: f32,
    pub stress_control_interval: u8,    // minutes
    pub decrease_alert_level_after: u8, // minutes
    pub suspend_timeout: u8,            // minutes
}

impl Default for WizardInfo {
    #[inline]
    fn default() -> WizardInfo {
        WizardInfo {
            last_stress_control_time: CtrlTime(0), //zero never updated
            daily_tgt_grass_et: DAILY_TGT_GRASS_ET,
            stress_control_interval: 6,
            decrease_alert_level_after: 11,
            suspend_timeout: 40,
        }
    }
}
