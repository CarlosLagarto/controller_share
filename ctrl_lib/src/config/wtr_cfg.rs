use rusqlite::Connection;
use std::str::FromStr;

use crate::data_structs::rega::{mode::*, state::*, wizard_info::*};
use crate::db::{db_error::*, db_sql_lite::*};
use crate::{app_time::ctrl_time::*, config::geo_pos::*, config::*};
use ctrl_prelude::domain_types::*;

/// Dimension = 88
pub struct WtrCfg {
    pub db: Persist,
    pub state: State,
    pub in_error: u8,
    pub in_alert: u8,
    pub mode: Mode,

    pub pump_recycle_time: u8,
    pub max_sector_time: DUR,

    pub wizard_info: WizardInfo,
    pub geo_pos: GeoPos,
    pub changed: bool,
    pub last_saved: CtrlTime,
    pub live_since: CtrlTime,
    pub last_stop: CtrlTime,
    pub fresh_start: u8, // 0 is fresh start, > 0 is not fresh start
}

impl WtrCfg {
    #[inline]
    pub fn new(db: Persist, start_up_time: CtrlTime) -> Self {
        let mut cfg = db.get_water_config(db.clone()).unwrap();
        cfg.live_since = start_up_time;
        cfg.changed = true;
        cfg
    }

    #[inline]
    pub fn save_if_updated(&mut self, time: CtrlTime) {
        if self.changed {
            self.last_saved = time;
            if self.db.save_water_config(self).is_ok() {
                self.changed = false; // do not need to handle error because is handled in the db call
            }
        }
    }
}

#[repr(u8)]
pub enum WateringParams {
    InError = 0,
    InAlert,
    State,
    Mode,
    PumpRecycleTime,
    MaxSectorTime,
    SuspendTimeout,
    DecreaseAlertLevelAfter,
    StressControlInterval,
    LastStressControlTime,
    DailyTgtGrassEt,
    LastSave,
    LiveSince,
    FreshStart,
}

pub trait ModelWaterConfig<'a>: ModelGeoPosConfig<'a> + DB {
    const GET_MODULE_WATER_CONFIG: &'a str = "SELECT float,int,string FROM mods_data where module=2 order by param;";
    const UPDATE_MODULE_WATER_CONFIG_STRING: &'a str = "update mods_data set string=?1 where module=2 and param=?2;";
    const UPDATE_MODULE_WATER_CONFIG_FLOAT: &'a str = "update mods_data set float=?1 where module=2 and param=?2;";
    const UPDATE_MODULE_WATER_CONFIG_INT: &'a str = "update mods_data set int=?1 where module=2 and param=?2;";

    #[inline]
    fn get_water_config(&self, db: Persist) -> Result<WtrCfg, DBError> {
        let geo_pos = self.get_geo_pos_config().unwrap();

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_MODULE_WATER_CONFIG).unwrap();
        let mut rows = stmt.raw_query();

        let mut wtr_cfg = WtrCfg {
            in_error: rows.next()?.unwrap().get_unwrap(DB_INT),
            in_alert: rows.next()?.unwrap().get_unwrap(DB_INT),
            state: State::from_str(&rows.next()?.unwrap().get_unwrap::<usize, String>(DB_STRING)).unwrap(),
            mode: Mode::from_str(&rows.next()?.unwrap().get_unwrap::<usize, String>(DB_STRING)).unwrap(),
            pump_recycle_time: rows.next()?.unwrap().get_unwrap(DB_INT),
            max_sector_time: rows.next()?.unwrap().get_unwrap(DB_INT),

            wizard_info: WizardInfo {
                suspend_timeout: rows.next()?.unwrap().get_unwrap(DB_INT),
                decrease_alert_level_after: rows.next()?.unwrap().get_unwrap(DB_INT),
                stress_control_interval: rows.next()?.unwrap().get_unwrap(DB_INT),
                last_stress_control_time: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_INT)),
                daily_tgt_grass_et: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            },
            last_saved: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_INT)),
            live_since: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_INT)),
            fresh_start: rows.next()?.unwrap().get_unwrap(DB_INT),
            last_stop: CtrlTime(0),
            db,

            geo_pos,
            changed: false,
        };
        wtr_cfg.last_stop = wtr_cfg.last_saved; // we can have a maximum 1 sec diff but 1for the wstering system, it doesn't matter
        drop(rows);
        drop(stmt);

        Ok(wtr_cfg)
    }

    #[inline]
    fn save_water_config(&self, cfg: &WtrCfg) -> SimpleResult {
        {
            let conn = &self.get_conn().conn;
            _ = self.update_u8(conn, WateringParams::InError as u8, cfg.in_error);
            
            _ = self.update_u8(conn, WateringParams::InAlert as u8, cfg.in_alert);
            _ = self.update_string(conn, WateringParams::State as u8, cfg.state.to_string());
            _ = self.update_string(conn, WateringParams::Mode as u8, cfg.mode.to_string());
            _ = self.update_u8(conn, WateringParams::PumpRecycleTime as u8, cfg.pump_recycle_time);
            _ = self.update_i64(conn, WateringParams::MaxSectorTime as u8, cfg.max_sector_time);
            _ = self.update_u8(conn, WateringParams::SuspendTimeout as u8, cfg.wizard_info.suspend_timeout);
            _ = self.update_u8(conn, WateringParams::DecreaseAlertLevelAfter as u8, cfg.wizard_info.decrease_alert_level_after);
            _ = self.update_u8(conn, WateringParams::StressControlInterval as u8, cfg.wizard_info.stress_control_interval);
            _ = self.update_u64(conn, WateringParams::LastStressControlTime as u8, cfg.wizard_info.last_stress_control_time.ux_ts());
            _ = self.update_f32(conn, WateringParams::DailyTgtGrassEt as u8, cfg.wizard_info.daily_tgt_grass_et);
            _ = self.update_u64(conn, WateringParams::LastSave as u8, cfg.last_saved.ux_ts());
            _ = self.update_u64(conn, WateringParams::LiveSince as u8, cfg.live_since.ux_ts());
            _ = self.update_u8(conn, WateringParams::FreshStart as u8, cfg.fresh_start);
        }
        self.save_geo_pos_config(&cfg.geo_pos)
    }

    #[inline]
    fn update_u8(&self, conn: &Connection, param: u8, val: u8) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WATER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_i64(&self, conn: &Connection, param: u8, val: i64) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WATER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_u64(&self, conn: &Connection, param: u8, val: u64) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WATER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_string(&self, conn: &Connection, param: u8, val: String) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WATER_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_f32(&self, conn: &Connection, param: u8, val: f32) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WATER_CONFIG_FLOAT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }
}

impl<'a> ModelWaterConfig<'a> for Persist {}
