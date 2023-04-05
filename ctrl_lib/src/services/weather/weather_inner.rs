use arrayvec::ArrayVec;
use ctrl_prelude::error::build_error;

use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::data_structs::msgs::{weather::*, alert::*, int_message::*};
use crate::data_structs::sensor::{daily_value::*, metrics::evapo_transpiracao::*, snsor::*, stat_metric::*};
use crate::{log_error, logger::*};
use crate::services::msg_broker::msg_brkr_svc::*;
use crate::services::weather::algorithms::*;
use crate::services::weather::rain_pred::{data_structs::*, naive_bayes::*};
use crate::services::weather::sources::simulation::mock_weather_data::*;
use crate::services::weather::sources::tempest::data_structs::BUF_SIZE;
use crate::services::weather::sources::{simulation::svc::*, tempest::rest::*, tempest::station::*};
use crate::services::weather::{db_model::*, weather_error::*, trend::*};
use crate::utils::TESTING;
use crate::{config::wthr_cfg::*, controller_sync::*, db::db_sql_lite::*};
use ctrl_prelude::domain_types::*;

pub const MAX_SOURCE_SWITCHES :u16 = 5;
pub const MAX_SOURCE_ISSUES:u16 = 10;
const COUNT_THRESHOLD_SENSOR:u16 = 1368;
const COUNT_THRESHLD_ALL_SENSORS: u16 = 10944;

/// Dimension = 3200
pub struct WeatherInner {
    pub db: Persist,

    pub station: WeatherStation,
    pub site: WeatherTempestRest,
    pub simulation: Simulation, // for UI client test

    pub sched_weather: Schedule,
    pub sched_new_day: Schedule,
    pub wthr_cfg: SWthrCfg, // Heap = 230

    pub bellow_threshold_since: UTC_UNIX_TIME,

    // try to reserve memory at startup to reuse during execution, avoiding some mallocs
    pub data: Vector<MAX_FEATURES>,

    pub(crate) sensor_data_buf: ArrayVec<SensorValue, MAX_SENSORS>,
    pub msg_broker: SMsgBrkr,

    pub weather_source: WeatherSource,
    pub o_weather: Option<Weather>,
    pub buf: Vec<u8>,
    pub station_altitude: f32,
    pub nr_of_gets_with_issues: u16,
    pub nr_of_sources_switch: u16,

    pub time_last_source_failure: CtrlTime,
    pub trend_data: TrendA,
}

impl WeatherInner {
    #[inline]
    pub fn new(time: CtrlTime, db: Persist, msg_broker: SMsgBrkr, wthr_cfg: SWthrCfg) -> Self {
        unsafe { MOCK_SIM = Some(MockSimulation::new()) };
        //we have 2 routines running daily. This one and db mnt, which starts in +10 secs
        let start_new_day = time.eod_ux_e() + NEW_DAY_START_DELAY; 
        let mut et_data_buffer = EtData::default();
        let weather_source: WeatherSource;
        let station_altitude: f32;

        {
            let wt = wthr_cfg.read();
            et_data_buffer.elev = wt.geo.elev;
            et_data_buffer.lat = wt.geo.lat;
            weather_source = wt.weather_source;
            station_altitude = wt.geo.elev as f32;
        }
        let update_interval = wthr_cfg.read().update_interval;

        let sched_wthr = Schedule::build_run_forever(time, update_interval, ScheduleRepeatUnit::Seconds);
        let sched_new_day = Schedule::build_run_forever(start_new_day, 1, ScheduleRepeatUnit::Days);
        Self {
            db,
            wthr_cfg,
            bellow_threshold_since: 0,

            sched_weather: sched_wthr,
            sched_new_day,

            data: Vector::new(),
            sensor_data_buf: ArrayVec::<SensorValue, MAX_SENSORS>::new(),
            station: WeatherStation {},
            site: WeatherTempestRest::new(),
            simulation: Simulation {},

            msg_broker,

            weather_source, 
            buf: vec![0_u8; BUF_SIZE], 
            station_altitude, 

            nr_of_gets_with_issues: 0, 
            nr_of_sources_switch: 0,

            o_weather: None,

            time_last_source_failure: CtrlTime(0),
            trend_data: TrendA::new(),
        }
    }

    /// Instantaneous eT doesn't make sense (very small numbers) - only a full day 
    /// - calc sensor's averages where applicable
    /// - calc last day temperature and relative humidity max & min
    /// - calc evapo transpiration of the previous day
    ///
    /// Also calculate the model
    #[inline]
    pub fn prep_new_day(&mut self, time: CtrlTime) -> Result<(), WeatherError> {
        let default_et: f32;
        let curr_ml_model: u32;
        let mut rain_class_forecast: f32 = 0.;
        // update local context
        {
            let mut wc_guard = self.wthr_cfg.write();
            // always update the day, despite having or not data
            wc_guard.set_current_day(time.as_date_char8_str_e());
            wc_guard.changed = true;
            default_et = wc_guard.default_et;
            curr_ml_model = wc_guard.current_ml_model;
        }
        let current_day = time.sod_ux_e();
        let prev_day = current_day.sub_days(1);
        // Get rain probability for the day
        // On error assumes zero
        self.data.clear();
        // This is the most used branch
        // Whet stop/starting on the same day, in short intervals, we may lost some data, but we assume that that do not affect statistis quality.
        // We either handle that here, or offline. After some considerations I decided to handle that offline.
        // Statistics are not relevant for daily running, so it's done this way, until we have a valid use case that support functional changes
        // 2023-02-11 - some tuning as the machine may stop/start during the day (p.e. maintenance), and still have statiscally relevant data for the day
        // Raw data is always in the db, so one can complement or access when desired.
        // Assuming (made up rule) 95% of the elements are enough, that imlies calculate daily data when minute record number > 1368 (95% of 1440).
        // In practice we have to analyse each sensor behaviour to enrich weather db data.  
        // To do waht?  Still unknown.  for now, just because.
        if let Ok((total, counts)) = self.db.get_daily_sensor_count(current_day){
            let mut daily_vec: [f64; MAX_FEATURES];
            // calculate last day daily data - we only have daily data with the previous assumptions

            // Data gaps, withj the previous assumptions, are beeing ignored.
            // SPRINT - coisas finais a fazer - podemos definir um threshold a partir do qual se considera que a qualidade dos dados no dia fica comprometida.  outra fase
            let elev = self.wthr_cfg.read().geo.elev;
            let lat = self.wthr_cfg.read().geo.lat;
            let mut rain_probability = 0.;
            daily_vec = self.db.get_minute_data_and_convert_to_daily_record(prev_day, elev, lat).unwrap();

            if total >= COUNT_THRESHLD_ALL_SENSORS { // thresholds ok
                // data manipulation for the prevision is offline - here we only register the daily data
                update_internal_data_all_ok(&mut self.data, daily_vec, prev_day);
                // and here we update the rain predicted class.  On any error acessing the BD its assumed as no rain
                // so lets predict the rain for the new day
                if let Some(pred) = get_rain_probability(&self.db, &daily_vec, curr_ml_model) {
                    daily_vec[Metric::RainClassForecast as usize] = pred.index as f64;
                    rain_probability = pred.rain_probability();
                } else {
                    daily_vec[Metric::RainClassForecast as usize] = 0.; // On error, no rain
                }
                rain_class_forecast = daily_vec[Metric::RainClassForecast as usize] as f32;
                self.data.push(SensorValue::new_rain_class_forecast(current_day, rain_class_forecast));
                self.data.push(SensorValue::new_rain_probability(current_day, rain_probability));
                // D Day is the ref
                // Forecast uses D-1 data (just computed daily vec )
                // D-1 real was forecasted in D-2
                let day_minus_2 = prev_day.sub_days(1);
                let real_rain_class_d_minus_2 = rain_class_from_rain_mm(daily_vec[Metric::SumRain as usize]) as f32;
                self.data.push(SensorValue::new_rain_class(day_minus_2, real_rain_class_d_minus_2));
            }else{
                update_internal_data_some_nok(&mut self.data, daily_vec, prev_day, counts);
                // Validate if we have enough data to calculate eT
                if !daily_vec[Metric::EvapoTranspiration as usize].is_nan(){
                    self.data.push(SensorValue::new_et(prev_day, daily_vec[Metric::EvapoTranspiration as usize] as f32));        
                }else{
                    // if not, use default
                    self.data.push(SensorValue::new_et(prev_day, default_et));        
                }
            }
        }else{
            // On error use default eT and continue
            self.data.push(SensorValue::new_et(prev_day, default_et));            
        }
        // // TODO: o tema de perceber quando o client context mudou
        //  In simulation mode, we can start at a time where there is already some info in the DB
        //  In production mode it's not supposed to happen so we will handle the error
        clear_sim_mode_prev_daily_data(&mut self.data, prev_day, &self.db);
        // and finally insert metrics in table, to explore latter, and evaluate prediction performance/eficiency
        _ = self.db.ins_daily_data_batch(&self.data);
        //     error!("Dados do erro no insert: \n{:?}", &self.data);
        // }
        {
            let mut config_lock = self.wthr_cfg.write();
            config_lock.rain_probability = rain_class_forecast;
            config_lock.save_if_updated(time); //save_if_updated already logs the error
        }
        Ok(())
    }

    #[inline]
    #[rustfmt::skip]
    pub fn process_manager(&mut self, time: CtrlTime) {
        let is_get_weather_time = self.sched_weather.is_time_to_run(time);
        let is_new_day = self.sched_new_day.is_time_to_run(time);
            
        if is_get_weather_time {
            // get weather data for all cases except udp from station
            match self.weather_source {
                WeatherSource::Station => (),  // already read
                // all other, get data - only timeout enters here
                WeatherSource::WebREST => self.o_weather = self.get_weather_from_site(time, self.station_altitude),
                WeatherSource::Simulation => self.o_weather = Some(self.simulation.get_weather(time))
            }
            self.process_new_day_if_its_time(is_new_day, time);
            if self.o_weather.is_some(){
                if self.nr_of_gets_with_issues > 0 && time > self.time_last_source_failure.add_secs(3600){
                    // if issues, try to get back to original source after 1 hour.  This may imply change source every hour if issue is not resolved
                    // but allows for living with intermitent failures
                    // intermitent failures may be seen on the log files that will allow for alert and further investigation
                    self.nr_of_gets_with_issues = 0;
                    _ = self.switch_source();
                }
                let weather = self.o_weather.as_mut().unwrap();
                // and now, do stuff with weather data
                // pressure variation velocity and derived data
                weather.pressure_velocity = self.trend_data.trend_analysis(weather.pressure as f64);

                self.sensor_data_buf.clear();
                self.sensor_data_buf.push(SensorValue::new(Sensor::Rain as u8, time, weather.rain_period));
                self.sensor_data_buf.push(SensorValue::new(Sensor::TempOutside as u8, time, weather.temperature));
                self.sensor_data_buf.push(SensorValue::new(Sensor::WindBearing as u8, time, weather.wind_bearing));
                self.sensor_data_buf.push(SensorValue::new(Sensor::WindSpeed as u8, time, weather.wind_intensity));
                self.sensor_data_buf.push(SensorValue::new(Sensor::HrOutside as u8, time, weather.humidity));
                self.sensor_data_buf.push(SensorValue::new(Sensor::Pressure as u8, time, weather.pressure));
                self.sensor_data_buf.push(SensorValue::new(Sensor::SolarRadiation as u8, time, weather.solar_rad));
                self.sensor_data_buf.push(SensorValue::new(Sensor::DewPoint as u8, time, dew_point(weather.temperature as f64, weather.humidity as f64) as f32));

                // update db

                //  In simulation mode, the program can start at a time where there is already some info in the DB
                //  In production mode that's not supposed to happen - thats an error that it is logged
                clear_sim_mode_prev_data(&mut self.sensor_data_buf, &self.db);

                if self.db.ins_sensor_data_batch(&self.sensor_data_buf).is_err() {
                    let msg = WeatherError::CantInsertDailyMeasures.to_string();
                    log_error!(msg);
                }
                // send data to listeners 
                self.msg_broker.reg_int_msg(MsgData::Weather(weather.clone()), time);
                // evaluate alerts
                {
                    let mut config_lock = self.wthr_cfg.write();
                    if config_lock.alrt_thresholds.is_rain_alert(weather.rain_period) {
                        snd_alert(AlertType::RAIN, weather.rain_period, time, &self.msg_broker);
                    }
                    if config_lock.alrt_thresholds.is_wind_alert(weather.wind_intensity) {
                        snd_alert(AlertType::WIND, weather.wind_intensity, time, &self.msg_broker);
                    }
                    config_lock.save_if_updated(time);
                }
                // advance to next event
                _ = self.sched_weather.set_next_event().map_err(|e| log_error!(build_error(&e)));
            } else {
                // no data....
                warn!("Não se conseguiu obter dados da metereologia na fonte: {}", self.weather_source);
                self.nr_of_gets_with_issues += 1;
                self.time_last_source_failure = time;  // record last failure time
                if self.nr_of_gets_with_issues >= MAX_SOURCE_ISSUES {
                    // MAX_SOURCE_ISSUES station failures (either intermitent or not) without data.
                    // change source (udp to rest or vice-versa)
                    let no_change = self.switch_source();
                    if !no_change {
                        warn!("Alterou-se temporariamente a fonte dos dados da metereologia para {}", self.weather_source);
                        self.nr_of_sources_switch += 1;
                    }
                    if self.nr_of_sources_switch >= MAX_SOURCE_SWITCHES {
                        let msg = format!("Mudou-se a fonte de dados da metereologia mais de {} vezes.  Validar o que se passa.",MAX_SOURCE_SWITCHES);
                        log_error!(msg);
                        self.msg_broker.snd_error_to_client(&msg);
                        self.nr_of_sources_switch = 0;  //switch counter reset
                    }
                }
            }
        }else{
            self.process_new_day_if_its_time(is_new_day, time);
        }
    }

    #[inline]
    pub fn get_weather_from_site(&self, time: CtrlTime, station_altitude: f32) -> Option<Weather> {
        let mut weather_result = None;
        if self.sched_weather.is_time_to_run(time) {
            let wc = self.wthr_cfg.read();
            if let Ok(w) = self.site.get_weather(time, wc.token_tempest.clone(), wc.device_id_tempest.clone(), station_altitude) {
                weather_result = Some(w);
            }
            // else, weather_result will return None, which will be handled accordingly 
        }else{
            weather_result = self.o_weather.clone();
        }
        weather_result
    }

    #[inline]
    fn process_new_day_if_its_time(&mut self, is_new_day: bool, time: CtrlTime) {
        if is_new_day{
            // handle new day before handling weather data
            new_day_and_db_mnt_sync();
            // wait for new day processing or db maint - at night
            NEW_DAY_SIG.read().reset();
            let res = self.prep_new_day(time);
            NEW_DAY_SIG.read().set();
            // advance to next day event.
            _ = self.sched_new_day.set_next_event().map_err(|e| log_error!(build_error(&e)));
            // On error notify clients
            res.unwrap_or_else(|e| { 
                let msg = e.to_string();
                log_error!(&msg);
                self.msg_broker.snd_error_to_client(&msg); 
            });
        }
    }

    #[inline]
    fn switch_source(&mut self)->bool {
        let mut no_change = false;
        let mut wc = self.wthr_cfg.write();
        match self.weather_source {
            WeatherSource::Station => self.weather_source = WeatherSource::WebREST,
            WeatherSource::WebREST => self.weather_source = WeatherSource::Station,
            _ => no_change = true,  // simulation data never fail (unless programming error)
        }
        wc.weather_source = self.weather_source;
        no_change
    }

}

#[inline]
pub fn update_internal_data_all_ok(result_vec: &mut Vector<MAX_FEATURES>, daily_vec: [f64; MAX_FEATURES], prev_day: CtrlTime) {
    for i in RAIN_CLASS_KEYS {
        if !daily_vec[i as usize].is_nan() {
            result_vec.push(SensorValue::new(i, prev_day, daily_vec[i as usize] as f32));
        }
    }
}

#[inline]
pub fn update_internal_data_some_nok(int_data: &mut Vector<MAX_FEATURES>, daily_vec: [f64; MAX_FEATURES], prev_day: CtrlTime, counts: Vec<SensorCount>) {
    let mut value: f32;
    let mut idx_helper: usize;
    let mut idx_map: usize;
    // some data nok.  Evaluate thresholds
    for sensor in counts{
        if sensor.count > COUNT_THRESHOLD_SENSOR{
            idx_helper = DERIVED_HELPER_MAP[sensor.sensor_id as usize];
            if idx_helper < usize::MAX{
                for j in 0..DERIVED_MAP_COUNT{
                    idx_map = DERIVED_MAP[idx_helper][j];
                    if idx_map < usize::MAX{
                        value = daily_vec[idx_map] as f32;
                        if !value.is_nan() && value < usize::MAX as f32 { 
                            int_data.push(SensorValue::new(sensor.sensor_id as u8, prev_day, value)); 
                        }
                    }
                }
            }
        }
        //else{}  // do nothing if bellow statistical ok threshold
    }
}

///  In simulation mode, we can start at a time where there is already some info in the DB
///
///  In production mode it's not supposed to happen so we will handle the error
#[inline]
#[rustfmt::skip]
pub fn clear_sim_mode_prev_daily_data(data: &mut Vector<MAX_FEATURES>, date_ref: CtrlTime, db: &Persist) {
    if !unsafe{TESTING} { return };
    // // to study why some times we have colision keys in DB
    // // In simulation mode, if there is already info for the period, we will just skip the data
    let aux_keys: Vec<u8> = data.idx.keys().cloned().collect();
    for metric in aux_keys {
        _ = db.get_daily_metric(metric, date_ref.ux_ts()).map(|measure| measure.map(|_| data.remove(unsafe { Metric::from_unchecked(metric) })));
    }
}

#[inline]
#[rustfmt::skip]
pub fn clear_sim_mode_prev_data<const CAP: usize>(data: &mut ArrayVec<SensorValue, { CAP }>, db: &Persist) {
    if !unsafe{TESTING} { return };
    // // to study why some times we have colision keys in DB
    // // In simulation mode, if there is already info for the period, we will just skip the data
    let mut row: &SensorValue;
    let mut i = 0;
    while i < data.len() {
        row = &data[i];
        if let Ok(measure) = db.get_sensor_data(row.id, row.timestamp.ux_ts()) {
            if measure.is_some() {
                data.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

#[inline]
pub fn snd_alert(alert_type: AlertType, value: f32, time: CtrlTime, msg_broker: &SMsgBrkr) {
    msg_broker.reg_int_msg(MsgData::Alert(Alert::new(alert_type, value)), time);
}
