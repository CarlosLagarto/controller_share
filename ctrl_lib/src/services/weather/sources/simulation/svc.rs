use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::weather::*;
use crate::services::weather::sources::simulation::mock_weather_data::{MockSim, MOCK_SIM};

/// Dimension 0
pub struct Simulation {}

impl Simulation {
    ///
    /// função auxiliar para quando tivermos a central, só se mexe aqui e o corpo principal não tem alterações.
    ///
    /// params:
    ///     - ts: time stamp para o qual e pretende o tempo
    ///
    #[inline]
    pub fn get_weather(&self, time: CtrlTime) -> Weather {
        // simulation for testing - mocking data for the client
        let x1 = unsafe { MOCK_SIM.next()};
        let x1_rads = x1.to_radians();
        let rand = || unsafe { MOCK_SIM.rand() };
        let rand_100 = || rand() * 100.;
        let get_x = || unsafe { MOCK_SIM.get_x() };

        Weather {
            current_time_ts: time.ux_ts(),
            utcnow_dt: time.as_date_web_str_e(),
            rain_period: rand(),
            rain_today: rand(),
            wind_bearing: x1 * get_x(),
            wind_intensity: (x1 % 180.) * get_x(),
            temperature: (unsafe { MOCK_SIM.get_temp() } + (x1_rads.sin().abs() * unsafe { MOCK_SIM.span_temp() } * get_x()).clamp(-16., 50.)),
            humidity: rand_100(),
            pressure: (unsafe { MOCK_SIM.get_press() } + (x1_rads.cos().abs() * unsafe { MOCK_SIM.span_press() } * get_x()).clamp(871., 1084.5)),
            rain_probability: rand_100(),
            ..Default::default()
        }
    }
}
