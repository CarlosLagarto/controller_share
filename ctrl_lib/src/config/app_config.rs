use crate::app_context::{db_mnt_cfg::*, start_up::*};
use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::config::{wtr_cfg::*, wthr_cfg::*, time_cfg::*, *};
use crate::data_structs::{client::client_ctx::*, msgs::alert_thresholds::*};
use crate::db::{db_error::*, db_sql_lite::*};
use crate::{log_error, logger::*};
use ctrl_prelude::{error::build_error, string_resources::*};

#[repr(u8)]
pub enum AppConfigParams {
    //appconfig
    CurrentDay = 0,
    LiveSince,
    LastSave,
    LastClientUpd,
    CheckClientInterval,
    ShutDown,
    FileSaveInterval,
    //start up
    StartDateStr,
    StartDate,
    //time data
    TimeControlInterval,
    TimeZone,
    //db maint
    DbMntCounter,
    DbMntLastRun,
    DbMntDays,
}

/// Dimension = 200
pub struct AppCfg {
    pub db: Persist,
    pub(crate) current_day: String, //      10 Heap
    pub(crate) schedule: Schedule,
    pub live_since: String,                  // "2020-08-13T05:01:45.690565000Z" + 30 Heap 40
    pub(crate) last_client_update: CtrlTime, //
    pub(crate) last_save: CtrlTime,
    pub changed: bool,
    pub(crate) check_client_interval: u8, //
    pub shutdown: u8,                     // 0 #se 1, foi não controlaado
    pub(crate) file_save_interval: u8,
    pub start_up: StartupData,
    pub time: TimeData, //   Heap 50
    pub db_maint: DBMntCfg,
}

impl AppCfg {
    #[inline]
    pub fn new(mut db: Persist) -> Self {
        db.get_app_config(db.clone()).unwrap()
    }

    /// analyse if the last shutdown was controlled (== 0).
    /// - on startup test if the value is 1, last shutdown was uncontrolled.
    /// - on startup put the flag as 1, and save the file.  If anything happens, next start will know that the shutdown was uncontrolled
    /// - on controlled shutdown put the flag == 0.
    #[inline]
    pub fn set_clean_shutdown(&mut self) -> u8 {
        self.shutdown = 1;
        self.changed = true;
        1
    }

    #[inline]
    pub fn set_shutdown_not_controlled(&mut self) -> u8 {
        self.shutdown = 0;
        self.changed = true;
        0
    }

    #[inline]
    pub const fn was_last_shutdown_controlled(&self) -> bool {
        self.shutdown == 1
    }

    #[inline]
    pub fn set_increment_db_maint_counter(&mut self) {
        self.db_maint.db_mnt_counter += 1;
        self.changed = true;
    }

    #[inline]
    pub fn reset_db_maint_counter(&mut self) {
        self.db_maint.db_mnt_counter = 0;
        self.changed = true;
    }

    #[inline]
    pub fn set_db_maint_last_run(&mut self, last_run: CtrlTime) {
        self.db_maint.db_mnt_last_run = last_run;
        self.changed = true;
    }

    #[inline]
    pub fn set_live_since(&mut self, value: CtrlTime) {
        self.live_since = value.as_rfc3339_str_e();
        self.changed = true;
    }

    #[inline]
    pub fn from_client(&mut self, cfg: &ClientCtx, config: &mut WtrCfg, wthr_cfg: &mut SWthrCfg) {
        self.live_since = cfg.live_since.to_string();
        self.changed = true;

        // water service
        config.in_error = cfg.in_error;
        config.in_alert = cfg.in_alert;
        config.state = cfg.current_state;
        config.mode = cfg.mode;

        config.pump_recycle_time = cfg.pump_recycle_time;
        config.max_sector_time = cfg.max_sector_time;
        let wi = &mut config.wizard_info;
        wi.stress_control_interval = cfg.stress_control_interval;
        wi.suspend_timeout = cfg.watering_suspend_timeout;
        wi.decrease_alert_level_after = cfg.decrease_alert_level_after;

        // weather service
        let mut wc = wthr_cfg.write();
        wc.alrt_thresholds = AlrtThresholds {
            rain: cfg.rain_alert_threshold,
            wind: cfg.wind_alert_threshold,
        };

        // db maint service
        self.db_maint.db_mnt_days = cfg.db_maint_days;
        self.db_maint.db_mnt_last_run = CtrlTime::from_ux_ts(cfg.db_maint_last_run);
    }

    #[inline]
    pub fn save_if_updated(&mut self, time: CtrlTime) {
        if self.changed {
            self.last_save = time;
            self.changed = false;
            self.db.clone().save_app_config(self).unwrap_or_else(|e| log_error!(err_saving_app_cfg(&build_error(&e))));
        }
    }

    #[inline]
    pub(crate) fn to_client(&self, wtr_cfg: &WtrCfg, wthr_cfg: SWthrCfg) -> ClientCtx {
        let lock_weather_config = wthr_cfg.read();
        let wi = &wtr_cfg.wizard_info;
        ClientCtx {
            live_since: self.live_since.clone(),
            last_change: self.last_save.ux_ts(),

            // water service
            in_error: wtr_cfg.in_error,
            in_alert: wtr_cfg.in_alert,
            current_state: wtr_cfg.state,
            mode: wtr_cfg.mode,
            pump_recycle_time: wtr_cfg.pump_recycle_time,
            max_sector_time: wtr_cfg.max_sector_time,

            stress_control_interval: wi.stress_control_interval,
            watering_suspend_timeout: wi.suspend_timeout,
            decrease_alert_level_after: wi.decrease_alert_level_after,

            // weather service
            rain_alert_threshold: lock_weather_config.alrt_thresholds.rain,
            wind_alert_threshold: lock_weather_config.alrt_thresholds.wind,

            // aqui está ok - maintenance service
            db_maint_days: self.db_maint.db_mnt_days,
            db_maint_last_run: self.db_maint.db_mnt_last_run.ux_ts(),
        }
    }
}

pub trait ModelConfig<'a>: DB {
    const GET_MODULE_CONFIG: &'a str = "SELECT int, string FROM mods_data where module=0 order by param;";
    const UPDATE_MODULE_CONFIG_INT: &'a str = "update mods_data set int=?1 where module=0 and param=?2;";
    const UPDATE_MODULE_CONFIG_STRING: &'a str = "update mods_data set string=?1 where module=0 and param=?2;";

    #[inline]
    fn get_app_config(&mut self, db: Persist) -> Result<AppCfg, DBError> {
        let conn = &db.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_MODULE_CONFIG).unwrap();
        let mut rows = stmt.raw_query();
        
        let mut cfg = AppCfg {
            current_day: rows.next()?.unwrap().get_unwrap(DB_INT),
            live_since: rows.next()?.unwrap().get_unwrap(DB_INT),
            last_save: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_FLOAT)),
            last_client_update: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_FLOAT)),
            check_client_interval: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            shutdown: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            file_save_interval: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            start_up: StartupData {
                start_date_str: rows.next()?.unwrap().get_unwrap(DB_INT),
                start_date: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_FLOAT)),
            },
            time: TimeData {
                time_control_interval: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
                timezone: rows.next()?.unwrap().get_unwrap(DB_INT),
            },
            db_maint: DBMntCfg {
                db_mnt_counter: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
                db_mnt_last_run: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_FLOAT)),
                db_mnt_days: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            },
            db: db.clone(),
            schedule: Schedule::default(),
            changed: false,
        };
        let file_save_interval = cfg.file_save_interval as u16;
        cfg.schedule = Schedule::build_run_forever(CtrlTime::sys_time(), file_save_interval, ScheduleRepeatUnit::Seconds);

        Ok(cfg)
    }

    #[inline]
    fn save_app_config(&mut self, cfg: &AppCfg) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::CurrentDay as u8);
        _ = stmt.raw_bind_parameter(1, &cfg.current_day);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::LiveSince as u8);
        _ = stmt.raw_bind_parameter(1, &cfg.live_since);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::LastSave as u8);
        _ = stmt.raw_bind_parameter(1, cfg.last_save.ux_ts());
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::LastClientUpd as u8);
        _ = stmt.raw_bind_parameter(1, cfg.last_client_update.ux_ts());
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::CheckClientInterval as u8);
        _ = stmt.raw_bind_parameter(1, cfg.check_client_interval);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::ShutDown as u8);
        _ = stmt.raw_bind_parameter(1, cfg.shutdown);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::FileSaveInterval as u8);
        _ = stmt.raw_bind_parameter(1, cfg.file_save_interval);
        _ = self.exec_prep(&mut stmt);

        //start up
        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::StartDateStr as u8);
        _ = stmt.raw_bind_parameter(1, &cfg.start_up.start_date_str);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::StartDate as u8);
        _ = stmt.raw_bind_parameter(1, cfg.start_up.start_date.ux_ts());
        _ = self.exec_prep(&mut stmt);

        //time data
        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::TimeControlInterval as u8);
        _ = stmt.raw_bind_parameter(1, cfg.time.time_control_interval);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::TimeZone as u8);
        _ = stmt.raw_bind_parameter(1, &cfg.time.timezone);
        _ = self.exec_prep(&mut stmt);

        //db maint
        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::DbMntCounter as u8);
        _ = stmt.raw_bind_parameter(1, cfg.db_maint.db_mnt_counter);
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::DbMntLastRun as u8);
        _ = stmt.raw_bind_parameter(1, cfg.db_maint.db_mnt_last_run.ux_ts());
        _ = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(2, AppConfigParams::DbMntDays as u8);
        _ = stmt.raw_bind_parameter(1, cfg.db_maint.db_mnt_days);
        self.exec_prep(&mut stmt)
    }
}

impl<'a> ModelConfig<'a> for Persist {}
