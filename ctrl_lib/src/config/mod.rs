pub mod _dev_notes;
pub mod app_config;
pub mod db_cfg;
pub mod geo_pos;
pub mod log_config;
pub mod mqtt_config;
pub mod time_cfg;
pub mod web_cfg;
pub mod wss_cfg;
pub mod wthr_cfg;
pub mod wtr_cfg;

#[repr(u8)]
pub enum Module {
    AppCfg = 0,
    MQTT = 1,
    Water = 2,
    GeoPos = 3,
    Weather = 4,
}

pub const DB_FLOAT: usize = 0;
pub const DB_INT: usize = 1;
pub const DB_STRING: usize = 2;