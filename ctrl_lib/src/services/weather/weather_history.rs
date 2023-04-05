use std::fmt;

use serde::{Deserialize, Serialize};

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{ext_message::*, topic::*};
use crate::db::db_sql_lite::*;
use crate::services::weather::{db_model::*, history_value::*, weather_error::*};
use ctrl_prelude::domain_types::*;

/// Dimension = 152
#[derive(Clone, Serialize, Deserialize)]
pub struct WeatherHstry {
    pub header: Option<Header>,
    pub temp_and_hp: Vec<DataArray>,
    pub end1: u64,
    pub wind: Vec<DataArray>,
    pub end2: u64,
    pub request_uuid: String,
}

impl fmt::Debug for WeatherHstry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeatherHstry")
            .field("header", &self.header)
            .field("temp_and_hp is of len ", &self.temp_and_hp.len())
            .field("end1", &self.end1)
            .field("wind is of len ", &self.wind.len())
            .field("end2", &self.end2)
            .field("request_uuid", &self.request_uuid)
            .finish()
    }
}
/// Dimension = 24
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataArray {
    diff: i64,
    val1: f64,
    val2: f64,
}

impl DataArray {
    #[inline]
    pub fn from_history(source: &HistoryValue) -> DataArray {
        DataArray {
            diff: source.diff,
            val1: source.val1,
            val2: source.val2,
        }
    }
}

impl WeatherHstry {
    #[inline]
    pub fn new_out(data: ExtMsgOut, time: CtrlTime) -> ExtMsgOut {
        ExtMsgOut::new(Topic::STC_WEATHER_HIST, data, time)
    }

    #[inline]
    pub fn build(time: UTC_UNIX_TIME, db: &Persist, request_uuid: String) -> Option<WeatherHstry> {
        let step1_result = last_24hrs_press_and_temp(time, db);
        let step2_result = last_24hrs_wind(time, db);
        if let (Ok((temp_and_hp, end1)), Ok((wind, end2))) = (step1_result, step2_result) {
            let data = WeatherHstry {
                header: None,
                temp_and_hp,
                end1,
                wind,
                end2,
                request_uuid,
            };
            Some(data)
        } else {
            None
        }
    }
}

#[inline]
fn last_auxiliar(rows: HistoryList) -> (Vec<DataArray>, u64) {
    let mut end: UTC_UNIX_TIME = 0;
    let n_rows = rows.len();
    if !rows.is_empty() {
        end = rows[n_rows - 1].minutets
    };
    let an_array = rows.iter().map(DataArray::from_history).collect();
    (an_array, end)
}

#[inline]
fn last_24hrs_press_and_temp(ts: UTC_UNIX_TIME, db: &Persist) -> WeatherResult<(Vec<DataArray>, u64)> {
    db.get_temp_pres_history(ts).map(last_auxiliar).map_err(|_| WeatherError::GettingPressureAndTemperatureHistory)
}

#[inline]
fn last_24hrs_wind(ts: UTC_UNIX_TIME, db: &Persist) -> WeatherResult<(Vec<DataArray>, u64)> {
    db.get_wind_history(ts).map(last_auxiliar).map_err(|_| WeatherError::GettingWindHistory)
}
