use serde::{self, Deserialize, Serialize};

use ctrl_prelude::domain_types::*;
use crate::data_structs::rega::{mode::*, state::*};

// This is data from the client so the names must be in sync with client json
/// Dimension 72
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientCtx {
    //General
    pub live_since: String, //UTC_ISO_DATE_STR
    pub last_change: UTC_UNIX_TIME,
    //StateMachineConfig
    pub in_error: u8,
    pub in_alert: u8,
    pub pump_recycle_time: u8,
    pub stress_control_interval: u8,
    pub watering_suspend_timeout: u8,
    pub max_sector_time: DUR,
    pub current_state: State,
    pub mode: Mode,
    //Meteo
    pub decrease_alert_level_after: u8,
    pub rain_alert_threshold: f32,
    pub wind_alert_threshold: f32,
    //Database
    pub db_maint_days: u8,
    pub db_maint_last_run: UTC_UNIX_TIME,
}

impl ClientCtx {
    #[inline]
    pub fn from_context(cfg: &ClientCtx) -> ClientCtx {
        ClientCtx {
            live_since: cfg.live_since.clone(),
            last_change: cfg.last_change,
            in_error: cfg.in_error,
            in_alert: cfg.in_alert,
            pump_recycle_time: cfg.pump_recycle_time,
            stress_control_interval: cfg.stress_control_interval,
            watering_suspend_timeout: cfg.watering_suspend_timeout,
            max_sector_time: cfg.max_sector_time,
            current_state: cfg.current_state,
            mode: cfg.mode,
            decrease_alert_level_after: cfg.decrease_alert_level_after,
            rain_alert_threshold: cfg.rain_alert_threshold,
            wind_alert_threshold: cfg.wind_alert_threshold,
            db_maint_days: cfg.db_maint_days,
            db_maint_last_run: cfg.db_maint_last_run,
        }
    }
}
