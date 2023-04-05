use std::fmt;

use num_enum::UnsafeFromPrimitive;
use rusqlite::Connection;

use crate::config::*;
use crate::data_structs::msgs::alert_thresholds::*;
use crate::db::{db_error::*, db_sql_lite::*};
use crate::{app_time::ctrl_time::*, config::geo_pos::*, utils::ArcRw};

///Dimension = 8
pub type SWthrCfg = ArcRw<WthrCfg>;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, UnsafeFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum WeatherSource {
    Station = 0,
    WebREST = 1,
    Simulation = 2, // for testing UI cliente REVIEW???
}

impl fmt::Display for WeatherSource {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WeatherSource::Station => write!(f, "Station"),
            WeatherSource::Simulation => write!(f, "Simulation"),
            WeatherSource::WebREST => write!(f, "Web REST"),
        }
    }
}

/// Dimension 224
pub struct WthrCfg {
    pub db: Persist,
    pub current_day: String, 
    pub rain_probability: f32,
    pub alrt_thresholds: AlrtThresholds,
    pub update_interval: u16,
    pub geo: GeoPos,
    pub last_save: CtrlTime,
    pub live_since: CtrlTime,
    pub changed: bool,
    pub default_et: f32,

    pub weather_source: WeatherSource,

    pub token_tempest: String,
    pub station_id_tempest: String,
    pub device_id_tempest: String,
    pub address_tempest: String,
    pub address_tempest_test: String,

    pub station_in_mnt: bool,
    pub current_ml_model: u32,
}

impl WthrCfg {
    #[inline]
    pub fn new(db: Persist, live_since: CtrlTime) -> Self {
        let mut wthr_cfg = db.get_weather_config(db.clone()).unwrap();
        wthr_cfg.live_since = live_since;
        wthr_cfg.current_day = CtrlTime::sys_time().as_date_char8_str_e();
        wthr_cfg
    }

    #[inline]
    pub fn save_if_updated(&mut self, time: CtrlTime) {
        if self.changed {
            self.last_save = time;
            _ = self.db.save_weather_config(self);
        }
    }

    #[inline]
    pub fn set_current_day(&mut self, current_day: String) {
        self.current_day = current_day;
        self.changed = true;
    }
}

#[derive(PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum WeatherParams {
    CurrentDay = 0,
    RainProbability = 1,
    UpdateInterval = 2,
    RainAlrtThreshold = 3,
    WindAlrtThreshold = 4,
    LastSave = 5,
    DailyTgtGrassEt = 6,
    TokenTempest = 7,
    StationIdTempest = 8,
    WeatherSource = 9,
    DeviceIdTempest = 10,
    AddressTempest = 11,
    CurrentMLModel = 12,
    AddressTempestTest = 13,
}

pub trait ModelWeatherConfig<'a>: ModelGeoPosConfig<'a> + DB {
    const GET_MODULE_WEATHER_CONFIG: &'a str = "SELECT float,int,string FROM mods_data where module=4 order by param;";
    const UPDATE_MODULE_WEATHER_CONFIG_STRING: &'a str = "update mods_data set string=?1 where module=4 and param=?2;";
    const UPDATE_MODULE_WEATHER_CONFIG_FLOAT: &'a str = "update mods_data set float=?1 where module=4 and param=?2;";
    const UPDATE_MODULE_WEATHER_CONFIG_INT: &'a str = "update mods_data set int=?1 where module=4 and param=?2;";

    #[inline]
    fn get_weather_config(&self, db: Persist) -> Result<WthrCfg, DBError> {
        let geo_pos: GeoPos;
        {
            geo_pos = self.get_geo_pos_config().unwrap();
        }
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_MODULE_WEATHER_CONFIG).unwrap();
        let mut rows = stmt.raw_query();

        let weather_cfg = WthrCfg {
            current_day: rows.next()?.unwrap().get_unwrap(DB_STRING),
            rain_probability: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            update_interval: rows.next()?.unwrap().get_unwrap(DB_INT),
            alrt_thresholds: AlrtThresholds {
                rain: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
                wind: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            },
            last_save: CtrlTime::from_ux_ts(rows.next()?.unwrap().get_unwrap(DB_INT)),
            geo: geo_pos,
            db,
            changed: false,
            live_since: CtrlTime(0), //é inicializado mais á frente
            default_et: rows.next()?.unwrap().get_unwrap(DB_FLOAT),
            token_tempest: rows.next()?.unwrap().get_unwrap(DB_STRING),
            station_id_tempest: rows.next()?.unwrap().get_unwrap(DB_STRING),
            weather_source: unsafe { WeatherSource::from_unchecked(rows.next()?.unwrap().get_unwrap(DB_INT)) },
            device_id_tempest: rows.next()?.unwrap().get_unwrap(DB_STRING),
            address_tempest: rows.next()?.unwrap().get_unwrap(DB_STRING),
            station_in_mnt: false, // este há-de ser mantido pelo UI do cliente.
            current_ml_model: rows.next()?.unwrap().get_unwrap(DB_INT),
            address_tempest_test: rows.next()?.unwrap().get_unwrap(DB_STRING),
        };

        drop(rows);
        drop(stmt);
        Ok(weather_cfg)
    }

    #[inline]
    fn save_weather_config(&self, cfg: &WthrCfg) -> SimpleResult {
        let mut _res2: Result<(), DBError>;
        {
            let conn = &self.get_conn().conn;
            _ = self.update_string(conn, WeatherParams::CurrentDay as u8, &cfg.current_day);
            _ = self.update_f32(conn, WeatherParams::RainProbability as u8, cfg.rain_probability);
            _ = self.update_u16(conn, WeatherParams::UpdateInterval as u8, cfg.update_interval);
            _ = self.update_f32(conn, WeatherParams::RainAlrtThreshold as u8, cfg.alrt_thresholds.rain);
            _ = self.update_f32(conn, WeatherParams::WindAlrtThreshold as u8, cfg.alrt_thresholds.wind);
            _ = self.update_f32(conn, WeatherParams::DailyTgtGrassEt as u8, cfg.default_et);
            _ = self.update_u64(conn, WeatherParams::LastSave as u8, cfg.last_save.ux_ts());
            _ = self.update_string(conn, WeatherParams::TokenTempest as u8, &cfg.token_tempest);
            _ = self.update_string(conn, WeatherParams::StationIdTempest as u8, &cfg.station_id_tempest);
            _ = self.update_u8(conn, WeatherParams::WeatherSource as u8, cfg.weather_source as u8);
            _ = self.update_string(conn, WeatherParams::DeviceIdTempest as u8, &cfg.device_id_tempest);
            _ = self.update_string(conn, WeatherParams::AddressTempest as u8, &cfg.address_tempest);
            _ = self.update_u32(conn, WeatherParams::CurrentMLModel as u8, cfg.current_ml_model);
            _ = self.update_string(conn, WeatherParams::AddressTempestTest as u8, &cfg.address_tempest_test);
        }
        self.save_geo_pos_config(&cfg.geo)
    }

    #[inline]
    fn update_u8(&self, conn: &Connection, param: u8, val: u8) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_u16(&self, conn: &Connection, param: u8, val: u16) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_u32(&self, conn: &Connection, param: u8, val: u32) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_u64(&self, conn: &Connection, param: u8, val: u64) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_INT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_string(&self, conn: &Connection, param: u8, val: &String) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_STRING).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn update_f32(&self, conn: &Connection, param: u8, val: f32) -> SimpleResult {
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_WEATHER_CONFIG_FLOAT).unwrap();
        _ = stmt.raw_bind_parameter(1, val);
        _ = stmt.raw_bind_parameter(2, param);
        self.exec_prep(&mut stmt)
    }
}

impl<'a> ModelWeatherConfig<'a> for Persist {}
