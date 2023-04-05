use crate::data_structs::sensor::{snsor::*, stat_metric::*};
use crate::{app_time::ctrl_time::*, db::SqlRow};

/// Dimension 16
#[derive(Clone, Copy)]
pub struct SensorCount {
    pub sensor_id: Sensor,
    pub count: u16,
}

impl SensorCount {
    #[inline]
    pub fn into_sensor_count(sql_row: &SqlRow) -> SensorCount {
        SensorCount {
            sensor_id: unsafe { Sensor::from_unchecked(sql_row.get(0).unwrap()) },
            count: sql_row.get(1).unwrap(),
        }
    }
}
/// Dimension 16
pub struct SensorValue {
    pub id: u8, // aqui tenho um u8 porque isto pode ser o Sensor:: ou uma StatMetric::
    pub timestamp: CtrlTime,
    pub value: f32,
}

impl SensorValue {
    #[inline]
    #[rustfmt::skip]
    pub const fn new(id: u8, timestamp: CtrlTime, value: f32) -> SensorValue {
        SensorValue { id, timestamp, value, }
    }
    #[inline]
    pub const fn new_et(timestamp: CtrlTime, value: f32) -> SensorValue {
        SensorValue::new(Metric::EvapoTranspiration as u8, timestamp, value)
    }

    #[inline]
    pub const fn new_rain_class(timestamp: CtrlTime, value: f32) -> SensorValue {
        SensorValue::new(Metric::RainClass as u8, timestamp, value)
    }

    #[inline]
    pub const fn new_rain_class_forecast(timestamp: CtrlTime, value: f32) -> SensorValue {
        SensorValue::new(Metric::RainClassForecast as u8, timestamp, value)
    }

    #[inline]
    pub const fn new_rain_probability(timestamp: CtrlTime, value: f32) -> SensorValue {
        SensorValue::new(Metric::RainProbability as u8, timestamp, value)
    }

    #[inline]
    pub fn into_sensor_data(sql_row: &SqlRow) -> SensorValue {
        SensorValue {
            id: sql_row.get(0).unwrap(),
            timestamp: CtrlTime(sql_row.get::<usize, f64>(1).unwrap().round() as u64),
            value: sql_row.get(2).unwrap(),
        }
    }
}
