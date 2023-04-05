use serde::{self, Deserialize, Serialize};

use ctrl_prelude::domain_types::*;

use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{mode::*, state::*};

// Isto vem do cliente, pelo que os nomes devem estar alinhados com o json que está e vem do cliente
/// Dimensão 104
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ClientContext {
    //General
    pub simulation: SIM,
    pub live_since: UTC_ISO_DATE_STR, //25
    pub last_change: CtrlTime,
    //Geo_Pos - não tem espelho/maintenance no client
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
    pub db_maint_last_run: CtrlTime,
    //Mqtt - não tem espelho/maintenance no client
    //WebServer  - não tem espelho/maintenance no client
}

impl ClientContext {
    #[inline]
    pub fn context_changed(&self, cfg: &ClientContext) -> bool {
        self.simulation != cfg.simulation
            || self.live_since != cfg.live_since
            || self.in_error != cfg.in_error
            || self.in_alert != cfg.in_alert
            || self.pump_recycle_time != cfg.pump_recycle_time
            || self.stress_control_interval != cfg.stress_control_interval
            || self.watering_suspend_timeout != cfg.watering_suspend_timeout
            || self.max_sector_time != cfg.max_sector_time
            || self.current_state != cfg.current_state
            || self.mode != cfg.mode
            || self.decrease_alert_level_after != cfg.decrease_alert_level_after
            || self.rain_alert_threshold != cfg.rain_alert_threshold
            || self.wind_alert_threshold != cfg.wind_alert_threshold
            || self.db_maint_days != cfg.db_maint_days
            || self.db_maint_last_run != cfg.db_maint_last_run
    }

    #[inline]
    pub fn from_context(cfg: &ClientContext) -> ClientContext {
        ClientContext {
            simulation: cfg.simulation,
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
