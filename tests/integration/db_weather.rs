use std::sync::{Arc};
use std::time::Instant;
// use parking_lot::RwLock;

use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::config::wthr_cfg::WthrCfg;
use ctrl_lib::data_structs::sensor::{daily_value::*, snsor::*, stat_metric::*};
use ctrl_lib::db::{db_error::*, db_sql_lite::*};
use ctrl_lib::services::msg_broker::msg_brkr_svc::MsgBrkr;
use ctrl_lib::services::weather::rain_pred::data_structs::Vector;
use ctrl_lib::services::weather::{algorithms::*, db_model::*, weather_inner::*};
use ctrl_lib::utils::{elapsed_dyn, arc_rw};
use ctrl_lib::ArrayVec;
use ctrl_prelude::domain_types::*;

pub trait DBModelWeatherTest<'a>: DBModelWeather<'a> {
    fn del_dly_measure(&self, sensor_id: SENSOR_ID_T, ts: DUR) -> SimpleResult {
        let sql: &str = "delete from daily_measure where id_sensor = ? and timestamp = ?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        let mut _res = stmt.raw_bind_parameter(1, sensor_id);
        let mut _res = stmt.raw_bind_parameter(2, ts);

        self.exec_prep(&mut stmt)
    }

    fn del_dly_avg_measure(&self, sensor_id: SENSOR_ID_T, date: &str) -> SimpleResult {
        let sql: &str = "delete from daily_measure_avg where id_sensor = ? and timestamp = ?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();

        let mut _res = stmt.raw_bind_parameter(1, sensor_id);
        let mut _res = stmt.raw_bind_parameter(2, date);
        self.exec_prep(&mut stmt)
    }

    fn del_sensor_data(&self, date_ref: CtrlTime) -> SimpleResult {
        let sql: &str = "delete from sensor_data where timestamp >= ? and timestamp <= ?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        let mut _res = stmt.raw_bind_parameter(1, date_ref.sod_ux_e().ux_ts());
        let mut _res = stmt.raw_bind_parameter(2, date_ref.eod_ux_e().ux_ts());
        self.exec_prep(&mut stmt)
    }

    fn del_daily_data(&self, date_ref: CtrlTime) -> SimpleResult {
        let sql: &str = "delete from sensor_daily_data where timestamp >= ? and timestamp <= ?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        let mut _res = stmt.raw_bind_parameter(1, date_ref.sod_ux_e().ux_ts());
        let mut _res = stmt.raw_bind_parameter(2, date_ref.eod_ux_e().ux_ts());
        self.exec_prep(&mut stmt)
    }

    fn del_all_sensor_daily_data(&self) -> SimpleResult {
        let sql: &str = "delete from sensor_daily_data;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn del_all_sensor_data(&self) -> SimpleResult {
        let sql: &str = "delete from sensor_data;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn degree(&self) -> SimpleResult {
        let sql: &str = "select degrees(3.14);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn radians(&self) -> SimpleResult {
        let sql: &str = "select radians(1);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn avg(&self) -> SimpleResult {
        let sql: &str = "select avg(1);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn atan2(&self) -> SimpleResult {
        let sql: &str = "select atan2(0.5, 0.5);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn sin(&self) -> SimpleResult {
        let sql: &str = "select sin(0.5);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn cos(&self) -> SimpleResult {
        let sql: &str = "select cos(0.5);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        self.exec_prep(&mut stmt)
    }

}

impl<'a> DBModelWeatherTest<'a> for Persist {}

const ITERS: u64 = 100;

#[test]
fn test_db_degree() {
    let db = Persist::new();
    let _result = db.degree();
    // assert!(result.is_ok());
}

#[test]
fn test_db_radians() {
    let db = Persist::new();
    let _result = db.radians();
    // assert!(result.is_ok());
}


#[test]
fn test_db_avg() {
    let db = Persist::new();
    let _result = db.avg();
    // assert!(result.is_ok());
}

#[test]
fn test_db_atan2() {
    let db = Persist::new();
    let _result = db.atan2();
    // assert!(result.is_ok());
}

#[test]
fn test_db_sin() {
    let db = Persist::new();
    let _result = db.sin();
    // assert!(result.is_ok());
}

#[test]
fn test_db_cos() {
    let db = Persist::new();
    let _result = db.cos();
    // assert!(result.is_ok());
}
#[test]
fn test_db_get_temp_pres_history() {
    let db = Persist::new();
    let result = db.get_temp_pres_history(CtrlTime(0).ux_ts());
    assert!(result.is_ok());
}

#[test]
fn db_get_temp_pres_history_time() {
    let db = Persist::new();

    let mut total = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.get_temp_pres_history(CtrlTime(0).ux_ts());
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_temp_pres_history_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn test_db_get_wind_history() {
    let db = Persist::new();
    let ux_ts = 1659318660;
    let result = db.get_wind_history(ux_ts);
    assert!(result.is_ok());
    let vals = result.unwrap();
    println!("len do vals: {}", vals.len());
    for val in vals{
        print!("{:<5.3}",val.minutets);
        print!(" ");
        print!("{:<5.3}",val.diff);
        print!(" ");
        print!("{:<5.3}",val.val1);
        print!(" ");
        print!("{:<5.3}",val.val2);
        println!();
    }
    
}

#[test]
fn db_get_wind_history_time() {
    let db = Persist::new();

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.get_wind_history(CtrlTime(0).ux_ts());
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_wind_history_time: {}", elapsed_dyn((total / ITERS) as u64));
}

#[rustfmt::skip]
#[test]
fn test_get_minute_data() {
    let (db, date_ref_original, date_ref, wc) = define_base_variables();

    _ = db.del_sensor_data(date_ref);

    create_one_day_of_data(date_ref, &db);

    let data = db.get_minute_data_and_convert_to_daily_record(date_ref_original, wc.geo.elev, wc.geo.lat);
    println!("{:?}", data);
    let oraculo = [50.0, 1013.0, 1.0, 50.0, 18.0, 50.0, 18.0, f64::NAN, 2.5663843154907227, 18.0, 1013.0, 1013.0, 1.0, 1.0, 1080.0, 0.0, 18.0, 18.0, 18.0, 18.0, 
                                 1013.0, 1013.0, 1013.0, 1013.0, 50.0, 50.0, 50.0, 50.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 
                                 7.420506000518799, 7.420506000518799, 7.420506000518799, 7.420506000518799, 7.420506000518799, 7.420506000518799, 
                                 7.420506000518799, 136.51360162355195,  0.0, 4.0, f64::NAN, 1440.0, 213.0];
    if let Ok(dados) = data{
        for (idx, val) in oraculo.iter().enumerate(){
            if !val.is_nan(){
                assert!((dados[idx] - val).abs() < 0.001);
            }else{
                assert!(dados[idx].is_nan());
            }
        }
    }else{
        assert!(false)
    }
}

#[test]
fn test_get_minute_data_time() {
    let (db, date_ref_original, date_ref, wc) = define_base_variables();

    _ = db.del_sensor_data(date_ref);

    create_one_day_of_data(date_ref, &db);
    let t = Instant::now();
    let _data = db.get_minute_data_and_convert_to_daily_record(date_ref_original, wc.geo.elev, wc.geo.lat);
    println!("db_get_minute_data_time:{}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));
    // println!("{:?}", _data);
}
pub fn define_base_variables() -> (Persist, CtrlTime, CtrlTime, WthrCfg) {
    let db = Persist::new();
    // temos que introduzir dados
    let date_ref_original = CtrlTime::from_utc_parts(2022, 08, 01, 0, 0, 0);
    let date_ref = date_ref_original;
    let wc = WthrCfg::new(db.clone(), date_ref);
    (db, date_ref_original, date_ref, wc)
}

#[rustfmt::skip]
pub fn create_one_day_of_data(mut date_ref: CtrlTime, db: &Persist) {
    let mut sensor_data_buf: ArrayVec<SensorValue, MAX_SENSORS> = ArrayVec::<SensorValue, MAX_SENSORS>::new();
    let mut sd: f32;
    for _hour in 0..24 {
        for _min in 0..60 {
            sensor_data_buf.clear();
            sensor_data_buf.push(SensorValue::new(Sensor::Rain as u8, date_ref, 1.));
            sensor_data_buf.push(SensorValue::new(Sensor::TempOutside as u8, date_ref, 18.));
            sensor_data_buf.push(SensorValue::new(Sensor::WindBearing as u8, date_ref, 0.));
            sensor_data_buf.push(SensorValue::new(Sensor::WindSpeed as u8, date_ref, 1.));
            sensor_data_buf.push(SensorValue::new(Sensor::HrOutside as u8, date_ref, 50.));
            sensor_data_buf.push(SensorValue::new(Sensor::Pressure as u8, date_ref, 1013.));            
            if (_hour > 9) & (_hour < 19) { sd = 120. } else { sd = 0. };
            sensor_data_buf.push(SensorValue::new(Sensor::SolarRadiation as u8, date_ref, sd));
            sensor_data_buf.push(SensorValue::new(Sensor::DewPoint as u8, date_ref, dew_point(18., 50.) as f32));

            // este não é preciso para o modelo, mas está aqui para ensair o código quando recebe info de outros sensores.
            sensor_data_buf.push(SensorValue::new(Sensor::WaterPumpCurrentDetection as u8, date_ref, 1013.));

            date_ref = date_ref.add_secs(60);
            _ = db.ins_sensor_data_batch(&sensor_data_buf);
        }
    }
}

#[test]
fn test_db_select_agregated_values() {
    let db = Persist::new();
    let result = db.get_daily_metric(0, CtrlTime(0).ux_ts());
    assert!(result.is_ok());
}

#[test]
fn db_select_agregated_values_time() {
    let db = Persist::new();

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.get_daily_metric(0, CtrlTime(0).ux_ts());
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_select_agregated_values_time: {}", elapsed_dyn((total / ITERS) as u64));
}

#[test]
fn test_db_get_daily_measure() {
    let db = Persist::new();
    let result = db.get_sensor_data(Sensor::TempOutside as u8, CtrlTime(0).ux_ts());
    assert!(result.is_ok());
}

#[test]
fn db_get_daily_measure_time() {
    let db = Persist::new();

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.get_sensor_data(Sensor::TempOutside as u8, CtrlTime(0).ux_ts());
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_daily_measure_time: {}", elapsed_dyn((total / ITERS) as u64));
}

#[test]
fn test_db_insert_sensor_data_batch() {
    let db = Persist::new();

    _ = db.del_all_sensor_daily_data();

    let mut sensor_rows = ArrayVec::<SensorValue, MAX_FEATURES>::new();
    let time = CtrlTime::sys_time();
    sensor_rows.push(SensorValue::new(Metric::SumRain as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgHumidity as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgPressure as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgWindSpeed as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::EvapoTranspiration as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MaxHumidity as u8, time, 0.));

    sensor_rows.push(SensorValue::new(Metric::MaxTemp as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MinHumidity as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MinTemp as u8, time, 0.));

    let result = db.ins_sensor_data_batch(&sensor_rows);
    assert!(result.is_ok());
}

#[test]
fn db_insert_sensor_data_batch_time() {
    let db = Persist::new();
    _ = db.del_all_sensor_daily_data();
    let mut sensor_rows = ArrayVec::<SensorValue, MAX_FEATURES>::new();
    let time = CtrlTime::sys_time();
    sensor_rows.push(SensorValue::new(Metric::SumRain as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgHumidity as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgPressure as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::AvgWindSpeed as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::EvapoTranspiration as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MaxHumidity as u8, time, 0.));

    sensor_rows.push(SensorValue::new(Metric::MaxTemp as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MinHumidity as u8, time, 0.));
    sensor_rows.push(SensorValue::new(Metric::MinTemp as u8, time, 0.));

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.ins_sensor_data_batch(&sensor_rows);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_db_insert_sensor_data_batch_time: {}", elapsed_dyn((total / ITERS) as u64));
}

//TODO - ver os testes que falta pensar no weather db

#[test]
fn create_daily_data() {
    let (db, date_ref_original, date_ref, wc) = define_base_variables();

    _ = db.del_sensor_data(date_ref);
    _ = db.del_daily_data(date_ref);

    create_one_day_of_data(date_ref, &db);

    let _data = db.get_minute_data_and_convert_to_daily_record(date_ref_original, wc.geo.elev, wc.geo.lat);

    if let Ok(data) = _data {
        let mut vector: Vector<MAX_FEATURES> = Vector::new();
        update_internal_data_all_ok(&mut vector, data, date_ref_original);
        let time = Instant::now();
        _ = db.ins_daily_data_batch(&vector);
        println!("db_ins_daily_data_batch: {}", elapsed_dyn((Instant::now() - time).as_nanos() as u64));
    }
}

#[test]
#[rustfmt::skip]
fn test_issue_prep_new_day_2() {
    let (db, _date_ref_original, _date_ref, wc) = define_base_variables();
    let smsg_broker = Arc::new(MsgBrkr::new());

    let swc = arc_rw(wc);
    let date_ref_original = CtrlTime::from_utc_parts(2023, 02, 11, 0, 0, 0);

    let mut wi = WeatherInner::new(date_ref_original.sub_days(2),db, smsg_broker, swc);

    let result = wi.prep_new_day(date_ref_original);
    println!("{:?}", result);
    assert!(result.is_ok());

}