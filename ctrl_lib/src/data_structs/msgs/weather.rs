use serde::{Deserialize, Serialize};

use ctrl_prelude::domain_types::*;

use super::ext_message::Header;

/// Dimension 144
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weather {
    pub header: Option<Header>,
    pub rain_period: f32, // weather station measures last minute rain
    pub rain_today: f32,
    pub rain_week_acc: f32,
    pub rain_probability: f32,
    pub rain_class_forecast: u8,
    pub wind_bearing: f32,
    pub wind_intensity: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub pressure_velocity: f32,
    pub current_time_ts: UTC_UNIX_TIME,
    pub utcnow_dt: UTC_ISO_DATE_STR,
    pub solar_rad: f32,
    pub et: f32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            header: None,
            rain_period: 0., // weather station measures last minute rain
            rain_today: 0.,
            rain_week_acc: 0.,
            rain_probability: 0.,
            rain_class_forecast: 0,
            wind_bearing: 0.,
            wind_intensity: 0.,
            temperature: 0.,
            humidity: 0.,
            pressure: 0.,
            pressure_velocity: 0.,
            current_time_ts: 0,
            utcnow_dt: "".to_owned(),
            solar_rad: 0.,
            et: 0.
        }
    }
}
