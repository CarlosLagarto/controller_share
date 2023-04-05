#![allow(clippy::derive_partial_eq_without_eq)]
use ctrl_prelude::domain_types::*;
use thiserror::*;

use ctrl_prelude::error::build_error;
use serde::{Deserialize, Serialize};

use crate::data_structs::msgs::weather::*;
use crate::lib_serde::{data_from_str, ConversionError};
use crate::services::weather::{algorithms::*, weather_error::*};
use crate::{app_time::ctrl_time::*, log_error, logger::*};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Status {
    pub status_code: u32,
    pub status_message: String,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Summary {
    pub pressure_trend: Option<String>,
    pub strike_count_1h: Option<u32>,
    pub strike_count_3h: Option<u32>,
    pub precip_total_1h: Option<f32>,
    pub feels_like: Option<f32>,
    pub heat_index: Option<f32>,
    pub wind_chill: Option<f32>,
}

/// Dimension 144
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TempestRest {
    ObsSt(ObsStRest),
    ObsAir(ObsAirRest),
    ObsSky(ObsSkyRest),
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsStObsRest {
    pub epoch: UTC_UNIX_TIME,          // 1635567982 Seconds
    pub wind_lull_min3: f32, // 0 Km
    pub wind_avg: f32,       // 0 Km
    pub wind_gust_max3: f32, // 0 Km
    pub wind_direction: u32, // 0 Km
    pub wind_sample_interval: u32,
    pub station_pressure: f32,
    pub air_temperature: f32, // degrees C
    pub relative_humidity: f32,
    pub illuminance: u32, // 835.0 MB
    pub uv: f32,          // 10.0 Degrees C
    pub solar_radiation: u32,
    pub rain_minute: f32,             // 45 %
    pub precipitation_type: u8,       // 0 = none, 1 = rain, 2 = hail, 3 = rain + hail
    pub average_strike_distance: u32, //km
    pub lightning_strike_count: u32,
    pub battery: f32,                                //volts
    pub report_interval: u32,                        // 1 Minutes
    pub local_day_rain_accumulation: f32,            //mm
    pub nc_rain_accumulation: Option<f32>,           //mm
    pub local_day_nc_rain_accumulation: Option<f32>, //mm
    pub precipitation_analysis_type: u32,            //0 = none, 1 = Rain Check with user display on, 2 = Rain Check with user display off
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsAirRest {
    pub epoch: UTC_UNIX_TIME, // 1635567982 Seconds
    pub station_pressure: f32,
    pub air_temperature: f32, // degrees C
    pub relative_humidity: f32,
    pub lightning_strike_count: u32,
    pub average_strike_distance: u32, //km
    pub battery: f32,                 //volts
    pub report_interval: u32,         // 1 Minutes
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsSkyRest {
    pub epoch: UTC_UNIX_TIME,             // 1635567982 Seconds
    pub illuminance: u32,       // 835.0 MB
    pub uv: f32,                // 10.0 Degrees C
    pub rain_accumulation: f32, // 45 %
    pub wind_lull_min3: f32,    // 0 Km
    pub wind_avg: f32,          // 0 Km
    pub wind_gust_max3: f32,    // 0 Km
    pub wind_direction: u32,    // 0 Km
    pub battery: f32,           //volts
    pub report_interval: u32,   // 1 Minutes
    pub solar_radiation: u32,
    pub local_day_rain_accumulation: f32, //mm
    pub precipitation_type: u8,           // 0 = none, 1 = rain, 2 = hail, 3 = rain + hail
    pub wind_sample_interval: u32,
    pub nc_rain_accumulation: Option<f32>,           //mm
    pub local_day_nc_rain_accumulation: Option<f32>, //mm
    pub precipitation_analysis_type: u32,            //0 = none, 1 = Rain Check with user display on, 2 = Rain Check with user display off
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsStRest {
    pub status: Status,
    pub device_id: u32,
    pub source: String,
    pub summary: Summary,
    pub obs: Vec<ObsStObsRest>,
}

#[derive(Debug, Error)]
pub enum WebError {
    #[error("Issue calling url: {0} with responde: {1}")]
    CallingURL(String, String),
    #[error("Issue processing get result: {0}, with error: {1}")]
    ProcessingResult(String, String),
    #[error("Issue deserializing result.")]
    UnknownSerializationIssue(),
}

pub type WebResult<T> = std::result::Result<T, WebError>;

pub struct WeatherTempestRest {
    pub last_obs_ts: UTC_UNIX_TIME,
}

impl WeatherTempestRest {
    #[allow(clippy::new_without_default)]
    #[inline]
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self { last_obs_ts: 0, }
    }

    #[inline]
    fn make_req(&self, url: &str) -> WebResult<ObsStRest> {
        match minreq::get(url).send() {
            Ok(body) => {
                let get_result = body.as_str().unwrap();
                data_from_str::<ObsStRest>(get_result).map_or_else(
                    |e| {
                        if let ConversionError::DeserializationIssue(a, b) = e {
                            Err(WebError::ProcessingResult(a, b))
                        } else {
                            unreachable!()
                        }
                    },
                    Ok,
                )
            }
            Err(err) => {
                let err = WebError::CallingURL(url.to_owned(), err.to_string());
                error!("{:?}", err);
                Err(err)
            }
        }
    }

    /// params:
    ///     - ts: timestamp of the request
    ///
    /// // TODO - explorar o api para obter os dados no periodo pretendido
    #[inline]
    pub fn get_weather(&self, time: CtrlTime, token: String, device_id: String, altitude: f32) -> Result<Weather, WeatherError> {
        let mut weather = Weather::default();

        let next_wthr: String = format!("https://swd.weatherflow.com/swd/rest/observations/?device_id={}&token={}", &device_id, &token);
        let res = self.make_req(&next_wthr);
        match res {
            Ok(val) => {
                let obs = &val.obs[0];
                // it may be a difference between the timestamp and the reported time. It is ignored.  
                // At most is one minute off, and I assume that the weather conditions change in one minute are small.
                weather.current_time_ts = time.ux_ts();
                weather.utcnow_dt = time.as_date_web_str_e();

                weather.rain_period = obs.rain_minute;
                weather.wind_bearing = obs.wind_direction as f32;
                weather.wind_intensity = obs.wind_avg;
                weather.temperature = obs.air_temperature;
                weather.humidity = obs.relative_humidity;
                weather.pressure = station_pressure_to_sea_pressure(obs.station_pressure, obs.air_temperature, altitude);
                Ok(weather)
            }
            Err(e) => {
                log_error!(&e); // conversion error
                log_error!(build_error(&WeatherError::GettingWeather));
                Err(WeatherError::GettingWeather)
            }
        }
    }
}
