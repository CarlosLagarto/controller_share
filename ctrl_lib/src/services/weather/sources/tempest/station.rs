use crate::services::weather::algorithms::station_pressure_to_sea_pressure;
use crate::services::weather::sources::tempest::data_structs::*;
use crate::{app_time::ctrl_time::*, data_structs::msgs::weather::*};

pub struct WeatherStation {}

impl WeatherStation {
    ///
    /// função auxiliar para quando tivermos a central, só se mexe aqui e o corpo principal não tem alterações.
    ///
    /// params:
    ///     - ts: time stamp para o qual e pretende o tempo
    ///
    #[inline]
    pub fn get_weather(&self, time: CtrlTime, obs: &ObsStObs, altitude: f32) -> Weather {
        Weather {
            current_time_ts: time.ux_ts(),
            utcnow_dt: time.as_date_web_str_e(),
            rain_period: obs.rain_minute,
            wind_bearing: obs.wind_direction as f32,
            wind_intensity: obs.wind_avg,
            temperature: obs.air_temperature,
            humidity: obs.relative_humidity,
            pressure: station_pressure_to_sea_pressure(obs.station_pressure, obs.air_temperature, altitude),
            ..Default::default()
        }
    }
}
