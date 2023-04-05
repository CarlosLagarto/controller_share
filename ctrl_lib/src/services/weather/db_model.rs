use crate::app_time::{ctrl_time::*, date_time::*, tm::*, sunrise::SECONDS_IN_A_DAY};
use crate::data_structs::sensor::{daily_value::*, stat_metric::*, snsor::*, metrics::evapo_transpiracao::*};
use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::weather::{history_value::*, rain_pred::data_structs::*};
use crate::{log_warn, logger::*};
use ctrl_prelude::domain_types::*;

const AGREGATE_COUNT: usize= 3;
const SUMARIZE_COUNT: usize = 5;
const HELPER_HOUR_BREAK_COUNT: usize = 4;

pub const DERIVED_MAP_COUNT: usize = 7;
pub const HELPER_DERIVED_MAP_COUNT: usize = 6;

#[repr(u8)]
enum MapSensor{
    Humidity = 0,
    Pressure = 1,
    Temp = 2,
    WindSpeed = 3,
    Dwp = 4,
    WindDirection = 5,
}

#[repr(usize)]
pub enum Sumarize{
    Avg = 0,
    Max = 1,
    Min = 2,
}

// Se mudar os enum dos sensores ou das métricas, tenho que rever aqui os indices.  Isto está hard coded para eficiência
#[rustfmt::skip]
pub const INI_HELPER: [[usize; HELPER_HOUR_BREAK_COUNT]; SUMARIZE_COUNT] = 
    [[Metric::AvgHumidity as usize,  Metric::MaxHumidity as usize,  Metric::MinHumidity as usize,  Metric::HrAt0 as usize], 
     [Metric::AvgPressure as usize,  Metric::MaxPressure as usize,  Metric::MinPressure as usize,  Metric::PressAt0 as usize],
     [Metric::AvgTemp as usize,      Metric::MaxTemp as usize,      Metric::MinTemp as usize,      Metric::TempAt0 as usize], 
     [Metric::AvgWindSpeed as usize, Metric::MaxWindSpeed as usize, Metric::MinWindSpeed as usize, Metric::WsAt0 as usize], 
     [Metric::AvgDwp as usize,       Metric::MaxDwp as usize,       Metric::MinDwp as usize,       Metric::DwpAt0 as usize] ];
                                    
/// este array mapeia para o array INI_HELPER
pub const SENSOR_BASE_MAP: [usize; MAX_SENSORS] = [usize::MAX, MapSensor::Temp as usize, MapSensor::Humidity as usize, MapSensor::WindSpeed as usize, usize::MAX, 
                                                MapSensor::Pressure as usize, usize::MAX, usize::MAX, MapSensor::Dwp as usize, usize::MAX];

pub const HOUR_HELPER :[[usize; HELPER_DERIVED_MAP_COUNT]; HELPER_HOUR_BREAK_COUNT] = 
       [[Metric::HrAt0 as usize,  Metric::PressAt0 as usize,  Metric::TempAt0 as usize,  Metric::WsAt0 as usize,  Metric::DwpAt0 as usize,  Metric::WdAt0 as usize],
        [Metric::HrAt6 as usize,  Metric::PressAt6 as usize,  Metric::TempAt6 as usize,  Metric::WsAt6 as usize,  Metric::DwpAt6 as usize,  Metric::WdAt6 as usize],
        [Metric::HrAt12 as usize, Metric::PressAt12 as usize, Metric::TempAt12 as usize, Metric::WsAt12 as usize, Metric::DwpAt12 as usize, Metric::WdAt12 as usize],
        [Metric::HrAt18 as usize, Metric::PressAt18 as usize, Metric::TempAt18 as usize, Metric::WsAt18 as usize, Metric::DwpAt18 as usize, Metric::WdAt18 as usize]];
                    
// este array mapeia para o array acima
pub const HOUR_HELPER_MAP :[usize; MAX_SENSORS] = [usize::MAX, MapSensor::Temp as usize, MapSensor::Humidity as usize, MapSensor::WindSpeed as usize, 
                                                    MapSensor::WindDirection as usize, MapSensor::Pressure as usize, usize::MAX, usize::MAX, 
                                                    MapSensor::Dwp as usize, usize::MAX];
// const x:Sensor =Sensor::DewPoint;
#[rustfmt::skip]
pub const DERIVED_MAP: [[usize; DERIVED_MAP_COUNT]; HELPER_DERIVED_MAP_COUNT] = 
    [[Metric::AvgHumidity as usize,  Metric::MaxHumidity as usize,  Metric::MinHumidity as usize, 
                Metric::HrAt0 as usize, Metric::HrAt6 as usize, Metric::HrAt12 as usize, Metric::HrAt18 as usize], 
     [Metric::AvgPressure as usize,  Metric::MaxPressure as usize,  Metric::MinPressure as usize,  
                Metric::PressAt0 as usize, Metric::PressAt6 as usize, Metric::PressAt12 as usize, Metric::PressAt18 as usize],
     [Metric::AvgTemp as usize,      Metric::MaxTemp as usize,      Metric::MinTemp as usize,      
                Metric::TempAt0 as usize, Metric::TempAt6 as usize, Metric::TempAt12 as usize, Metric::TempAt18 as usize], 
     [Metric::AvgWindSpeed as usize, Metric::MaxWindSpeed as usize, Metric::MinWindSpeed as usize, 
                Metric::WsAt0 as usize, Metric::WsAt6 as usize, Metric::WsAt12 as usize, Metric::WsAt18 as usize], 
     [Metric::AvgDwp as usize,       Metric::MaxDwp as usize,       Metric::MinDwp as usize,       
                Metric::DwpAt0 as usize, Metric::DwpAt6 as usize, Metric::DwpAt12 as usize, Metric::DwpAt18 as usize],
     [Metric::AvgWindDirection as usize, usize::MAX, usize::MAX, 
                Metric::WdAt0 as usize, Metric::WdAt6 as usize, Metric::WdAt12 as usize, Metric::WdAt18 as usize]]
;
                
pub const DERIVED_HELPER_MAP :[usize; MAX_SENSORS] = [usize::MAX, MapSensor::Temp as usize, MapSensor::Humidity as usize, 
                MapSensor::WindSpeed as usize, MapSensor::WindDirection as usize, MapSensor::Pressure as usize, usize::MAX, usize::MAX, 
                MapSensor::Dwp as usize, usize::MAX];

// Se mudar os enum dos sensores ou das métricas, tenho que rever aqui os indices.  Isto está hard coded para eficiência
// por convenção no código os indices são 0= Average, 1=Max, 2=Min
#[rustfmt::skip]
pub const UPD_HELPER: [[usize; AGREGATE_COUNT]; SUMARIZE_COUNT] = [
    [Metric::AvgHumidity as usize,  Metric::MaxHumidity as usize,  Metric::MinHumidity as usize], 
    [Metric::AvgPressure as usize,  Metric::MaxPressure as usize,  Metric::MinPressure as usize],
    [Metric::AvgTemp as usize,      Metric::MaxTemp as usize,      Metric::MinTemp as usize], 
    [Metric::AvgWindSpeed as usize, Metric::MaxWindSpeed as usize, Metric::MinWindSpeed as usize], 
    [Metric::AvgDwp as usize,       Metric::MaxDwp as usize,       Metric::MinDwp as usize] ];
                                    
pub trait DBModelWeather<'a>: DB {
    const HISTORY_SELECT_PRESS_AND_TEMP: &'a str = "select t1.minutets,?1-t1.minutets as diff, \
                                                coalesce(avg(case when id_sensor=1 then avg_value end),0) as val1,\
                                                coalesce(avg(case when id_sensor=5 then avg_value end),0) as val2 \
                                            from (SELECT id_sensor,cast(timestamp/60 as int) as minutets,avg(value) as avg_value \
                                            FROM sensor_data inner JOIN sensor ON id_sensor=sensor.id \
                                            where timestamp>?2 and timestamp<=?3 and (id_sensor=1 or id_sensor=5)\
                                            group by id_sensor,minutets order by id_sensor,minutets) as t1 group by t1.minutets \
                                            order by t1.minutets;";
     
    const HISTORY_SELECT_WIND: &'a str = "select T1.minutets, ?1-t1.minutets AS diff, val1, val2 from (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,\
                                AVG(value) AS val1 \
                                FROM sensor_data WHERE timestamp>=?2 AND timestamp<=?3 AND id_sensor=3 \
                                GROUP BY minutets ) T1 INNER JOIN \
                                    (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,\
                                        (degrees(atan2(AVG(sin(radians(value))),AVG(cos(radians(value)))))+360)%360 as val2 \
                                        FROM sensor_data WHERE timestamp>?2 AND timestamp<=?3 AND  id_sensor=4 \
                                        GROUP BY minutets) T2 ON T1.minutets = T2.minutets order by T1.minutets;";

    const SENSOR_DATA_SELECT: &'a str = "select id_sensor,value,timestamp from sensor_data where id_sensor=? and timestamp=?;";
    const SENSOR_DATA_INSERT: &'a str = "INSERT INTO sensor_data(id_sensor,timestamp,value)VALUES(?,?,?);";
    const DAILY_DATA_SELECT: &'a str = "select id_metric,value,timestamp from sensor_daily_data where id_metric=? and timestamp=?;";
    const DAILY_DATA_INSERT: &'a str = "INSERT INTO sensor_daily_data(id_metric,timestamp,value)VALUES(?,?,?);";

    const DAILY_DATA_COUNT: &'a str = "select id_sensor, count(*) as count from sensor_data where timestamp>? and timestamp<? group by id_sensor";

    const GET_SENSOR_MINUTE_VALUES: &'a str = "SELECT id_sensor,value,timestamp \
                                                FROM sensor_data where timestamp>? and timestamp<=? and id_sensor in(0,1,2,3,4,5,6,8)\
                                                ORDER BY timestamp,id_sensor;";

    const ML_SELECT_ML_MODEL: &'a str = "select model from ml_model_data where current_model_id=?;";

    const GET_RAIN_BETWEEN: &'a str = "SELECT coalesce(sum(value),0)as total from sensor_data where timestamp>=? and timestamp<=? and id_sensor=0;"; //Sabemos que a chuva é o sensor 0
    const GET_ET_BETWEEN: &'a str = "SELECT coalesce(sum(value),0)as total from sensor_daily_data where timestamp>=? and timestamp<? and id_metric=8;"; // sabemos que o ET é o id 8

    #[inline]
    fn get_temp_pres_history(&self, timestamp: UTC_UNIX_TIME) -> Result<HistoryList, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt =conn.prepare_cached(Self::HISTORY_SELECT_PRESS_AND_TEMP).unwrap();
        self.get_history(timestamp, &mut stmt)
    }

    #[inline]
    fn get_wind_history(&self, timestamp: UTC_UNIX_TIME) -> Result<HistoryList, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::HISTORY_SELECT_WIND).unwrap();
        self.get_history(timestamp,&mut stmt)
    }

    #[inline]
    fn get_history(&self, timestamp: UTC_UNIX_TIME, stmt: &mut rusqlite::CachedStatement) -> Result<HistoryList, DBError> {
        
        let minute_end = sec_to_min(timestamp as f32).ceil();
        let end = min_to_sec_f32(minute_end);
        let start = end - SECONDS_IN_A_DAY as f32;
        _ = stmt.raw_bind_parameter(1, minute_end);
        _ = stmt.raw_bind_parameter(2, start);
        _ = stmt.raw_bind_parameter(3, end);
        
        let mut history_values: HistoryList = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(row) = rows.next()? {
            history_values.push(HistoryValue::from_db_row(row));
        }
        Ok(history_values)
    }

    #[inline]
    fn get_model(&self, curr_model: u32) -> Result<Option<String>, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::ML_SELECT_ML_MODEL).unwrap();
        _ = stmt.raw_bind_parameter(1, curr_model);

        let mut rows = stmt.raw_query();

        if let Some(row) = rows.next()? {
            let model_str = row.get(0).unwrap();
            Ok(Some(model_str))
        } else {
            //pessegada porque não se obteve registos na query e deviamos
            log_warn!("Não foi encontrado nenhum modelo na tabela ml_model_data");
            Ok(None)
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn get_minute_data_and_convert_to_daily_record(&self, date_ref: CtrlTime, elev: f64, lat: f64) -> Result<DSRow<MAX_FEATURES>, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_SENSOR_MINUTE_VALUES).unwrap();

        let start = date_ref.sod_ux_e().ux_ts();
        let end = date_ref.eod_ux_e().ux_ts();

        _ = stmt.raw_bind_parameter(1, start);
        _ = stmt.raw_bind_parameter(2, end);
        // Isto devolve, se houver dados completos, 60 linhas * 24 horas * 8 sensores = 11520 registos - vamos ver a performance
        let mut rows = stmt.raw_query();

        let mut d: DateTimeE;

        let mut ds_first: [bool; MAX_SENSORS] = [true; MAX_SENSORS];
        let mut ds_hour: [i8; MAX_SENSORS] = [-1; MAX_SENSORS];
        let mut ds_row: DSRow<MAX_FEATURES> = [f64::NAN; MAX_FEATURES];
        let mut ds_count: [u16; MAX_SENSORS] = [0; MAX_SENSORS];
        let mut ds_sensor: [bool; MAX_SENSORS] = [false; MAX_SENSORS];

        let mut sum_wd_sin = 0.;
        let mut sum_wd_cos = 0.;

        let mut solar_rad = 0.;
        let mut idx;

        let mut sensor_id: usize;
        let mut timestamp: CtrlTime;
        let mut value: f64;
        let len_wd: f64;
        let et_data: EtData;
        // nos sensores com médias, usamos a metrica avg como temporaria, para guardar a soma, até dividir no fim pelo count respetivo
        while let Some(row) = rows.next()? {
            sensor_id = row.get(0).unwrap();
            timestamp = CtrlTime::from_ux_ts(row.get::<usize, f64>(2).unwrap().floor() as u64);
            value= row.get(1).unwrap();

            d = timestamp.as_utc_date_time_e();
            if ds_first[sensor_id] {
                // executa isto no primeiro registo para cada sensor
                ds_first[sensor_id] = false;
                ds_hour[sensor_id] = d.hour as i8;

                incr_sensor_count(sensor_id, &mut ds_count, &mut ds_sensor);

                match unsafe { Sensor::from_unchecked(sensor_id as u8) } {
                    Sensor::HrOutside | Sensor::Pressure | Sensor::TempOutside | Sensor::WindSpeed | Sensor::DewPoint => {
                        idx = SENSOR_BASE_MAP[sensor_id];
                        for i in INI_HELPER[idx]{
                            ds_row[i] = value;
                        }
                    },
                    Sensor::Rain => {
                        ds_row[Metric::SumRain as usize] = value;
                    }
                    Sensor::SolarRadiation => {
                        ds_row[Metric::SolarRadiation as usize] = 0.;
                    }
                    Sensor::WindBearing => {
                        ds_row[Metric::WdAt0 as usize] = value;
                        sum_wd_sin = f64::sin(value.to_radians());
                        sum_wd_cos = f64::cos(value.to_radians());
                    }
                    //este são os sensores 7 e 9, e eventualmente maiores que 9, que não estão no stmt sql, e que não ointeressam p+ara a metereologia
                    _ => (),  //nop  
                }
            }
            if ds_hour[sensor_id] != d.hour as i8 {
                // mudamos de hora e vamos fazer coisas
                if sensor_id == (Sensor::SolarRadiation as usize) {
                    if !ds_row[Metric::SolarRadiation as usize].is_nan() { 
                        ds_row[Metric::SolarRadiation as usize] += solar_rad / ds_count[sensor_id] as f64; 
                    }
                    solar_rad = 0.;  // reset para a média horária
                    ds_count[sensor_id] = 0; // reset do counter do nr leituras na hora
                }else{
                    match d.hour {
                        0 | 6 | 12 | 18 => {
                            match unsafe { Sensor::from_unchecked(sensor_id as u8) }{
                                Sensor::HrOutside | Sensor::Pressure | Sensor::TempOutside | Sensor::WindBearing | Sensor::WindSpeed | Sensor::DewPoint => {
                                    idx = HOUR_HELPER_MAP[sensor_id];
                                    ds_row[HOUR_HELPER[(d.hour / 6) as usize][idx]] = value;                
                                },
                                _ => (), //nop
                            }
                        },
                        _ => (), //nop
                    }
                }
                ds_hour[sensor_id] = d.hour as i8;
                incr_sensor_count(sensor_id, &mut ds_count, &mut ds_sensor);
                update_vec(&mut ds_row, &mut sum_wd_sin, &mut sum_wd_cos, sensor_id, value, &mut solar_rad);
            } else {
                incr_sensor_count(sensor_id, &mut ds_count, &mut ds_sensor);
                update_vec(&mut ds_row, &mut sum_wd_sin, &mut sum_wd_cos, sensor_id, value, &mut solar_rad);
            }
        } // fim do loop - processamos os registos todos
        // calculamos as médias - isto podia-se fazer uns loops aqui com uns arrays auxiliares, mas estou basicamente a fazer o loop unrolling
        if !ds_row[Metric::AvgHumidity as usize].is_nan() { ds_row[Metric::AvgHumidity as usize] /= ds_count[Sensor::HrOutside as usize] as f64; }
        if !ds_row[Metric::AvgPressure as usize].is_nan() { ds_row[Metric::AvgPressure as usize] /= ds_count[Sensor::Pressure as usize] as f64; }
        if !ds_row[Metric::AvgTemp as usize].is_nan() { ds_row[Metric::AvgTemp as usize] /= ds_count[Sensor::TempOutside as usize] as f64; }
        if !ds_row[Metric::AvgWindSpeed as usize].is_nan() { ds_row[Metric::AvgWindSpeed as usize] /= ds_count[Sensor::WindSpeed as usize] as f64; }
        if !ds_row[Metric::AvgDwp as usize].is_nan() { ds_row[Metric::AvgDwp as usize] /= ds_count[Sensor::DewPoint as usize] as f64; }
        if ds_sensor[Sensor::WindBearing as usize] {
            len_wd = ds_count[Sensor::WindBearing as usize] as f64;
            ds_row[Metric::AvgWindDirection as usize] = f64::to_degrees((f64::atan2(sum_wd_sin / len_wd, sum_wd_cos / len_wd) + 360.) % 360.);
        }
        if !ds_row[Metric::SumRain as usize].is_nan(){
            ds_row[Metric::RainClass as usize] = rain_class_from_rain_mm(ds_row[Metric::SumRain as usize]);
        }
        
        if EtData::valid_et_data(&ds_row) {
            et_data = EtData::from_ds_row(&ds_row, elev, lat, date_ref);
            ds_row[Metric::EvapoTranspiration as usize] = daily_evapo_transpiration(et_data) as f64;
        }
        if !ds_row[Metric::AvgPressure as usize].is_nan() && !ds_row[Metric::AvgDwp as usize].is_nan() && !ds_row[Metric::AvgHumidity as usize].is_nan() 
        {
            ds_row[Metric::PressureDwpRatio as usize] = ds_row[Metric::AvgPressure as usize] / ds_row[Metric::AvgDwp as usize];
            ds_row[Metric::HumidityGtERatio as usize] = ((ds_row[Metric::AvgHumidity as usize] > ds_row[Metric::PressureDwpRatio as usize]) as u8) as f64;
        }

        ds_row[Metric::DayNr as usize] = date_ref.year_day_number_e() as f64;

        // println!("Passagem 1 por {} linhas de um maximo em runtime de 11520 - leitura das medidas minuto a minuto", ds_count.iter().sum::<u16>());
        Ok(ds_row)
    }

    // Isto só existe para gerir colisões durante os testes
    #[inline]
    fn get_sensor_data(&self, id_sensor: SENSOR_ID_T, timestamp: UTC_UNIX_TIME) -> Result<Option<SensorValue>, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SENSOR_DATA_SELECT).unwrap();
        let mut measures: Option<SensorValue> = None;

        _ = stmt.raw_bind_parameter(1, id_sensor);
        _ = stmt.raw_bind_parameter(2, timestamp);
        let mut rows = stmt.raw_query();
        if let Some(row) = rows.next()? {
            measures = Some(SensorValue::into_sensor_data(row));
        }
        Ok(measures)
    }

    // Isto só existe para gerir colisões durante os testes
    #[inline]
    fn get_daily_metric(&self, id_sensor: SENSOR_ID_T, timestamp: UTC_UNIX_TIME) -> Result<Option<SensorValue>, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::DAILY_DATA_SELECT).unwrap();
        let mut measures: Option<SensorValue> = None;

        _ = stmt.raw_bind_parameter(1, id_sensor);
        _ = stmt.raw_bind_parameter(2, timestamp);
        let mut rows = stmt.raw_query();
        if let Some(row) = rows.next()? {
            measures = Some(SensorValue::into_sensor_data(row));
        }
        Ok(measures)
    }

    /// Num determinado time de referencia, vai buscar as contagens dos sensores para o dia anterior D-1
    /// O tempo de referencia assume-se que é o dia D.
    /// Assume-se ainda que já é o inicio (sod do dia D)
    /// // devolve um tupple (u16, counts vec) com o 1º elemento = total de registos existentes, e o 2º com os parciais
    #[inline]
    fn get_daily_sensor_count(&self, time: CtrlTime) -> Result<(u16, Vec<SensorCount>), DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::DAILY_DATA_COUNT).unwrap();
        let mut counts: Vec<SensorCount> = Vec::with_capacity(MAX_SENSORS);
        let ini = time.sub_days(1).ux_ts();
        let fim = ini + CtrlTime::SECS_IN_A_DAY;
        let mut total: u16 = 0;

        _ = stmt.raw_bind_parameter(1, ini);
        _ = stmt.raw_bind_parameter(2, fim);
        let mut rows = stmt.raw_query();
        while let Some(row) = rows.next()? {
            counts.push(SensorCount::into_sensor_count(row));
            total += counts[counts.len()-1].count;
        }
        Ok((total, counts))
    }

    #[inline]
    fn ins_sensor_data_batch(&self, rows: &[SensorValue]) -> SimpleResult {
        self.ins_data_batch(rows, Self::SENSOR_DATA_INSERT)
    }

    #[inline]
    fn ins_daily_data_batch(&self, rows: &Vector<MAX_FEATURES>) -> SimpleResult {
        let conn = &self.get_conn().conn;
        // println!("autocmmit no ins_daily_data_batch{:?}", conn.is_autocommit());
        let mut stmt = conn.prepare_cached(Self::DAILY_DATA_INSERT).unwrap();
        let mut res: SimpleResult = Ok(());
        let mut idx :Option<&usize>;
        for k in rows.idx.keys() {
            idx = rows.idx.get(k);
            if let Some(i) = idx {
                let sensor_data = &rows.data[*i];
                _ = stmt.raw_bind_parameter(1, sensor_data.id);
                _ = stmt.raw_bind_parameter(2, sensor_data.timestamp.ux_ts());
                _ = stmt.raw_bind_parameter(3, sensor_data.value);
                res = self.exec_prep(&mut stmt);
                if res.is_err(){
                    error!("Erro no insert da tabela sensor_daily_data. sensor id ={}, timestamp = {}, value = {} ", sensor_data.id, sensor_data.timestamp.as_rfc3339_str_e(), sensor_data.value);
                }
            }
        }
        res
    }

    #[inline]
    fn ins_data_batch(&self, rows: &[SensorValue], sql: &str) -> SimpleResult {
        let conn = &self.get_conn().conn;
        // println!("autocmmit no ins_data_batch{:?}", conn.is_autocommit());
        let mut stmt = conn.prepare_cached(sql).unwrap();
        let mut res: SimpleResult = Ok(());
        for row in rows {
            _ = stmt.raw_bind_parameter(1, row.id);
            _ = stmt.raw_bind_parameter(2, row.timestamp.ts());
            _ = stmt.raw_bind_parameter(3, row.value);
            res = self.exec_prep(&mut stmt);
        }
        res
    }

    // Assume que devolve sempre uma linha, com uma coluna apenas
    #[inline]
    fn get_rain_between(&self, time1: CtrlTime, time2: CtrlTime) -> Option<f32> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_RAIN_BETWEEN).unwrap();
        _ = stmt.raw_bind_parameter(1, time1.ux_ts());
        _ = stmt.raw_bind_parameter(2, time2.ux_ts());

        let mut rows = stmt.raw_query();
        get_row_val_f32(rows.next())
    }
    
            // Assume que devolve sempre uma linha, com uma coluna apenas
    #[inline]
    fn get_et_between(&self, time1: CtrlTime, time2: CtrlTime) -> Option<f32> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_ET_BETWEEN).unwrap();
        _ = stmt.raw_bind_parameter(1, time1.ux_ts());
        _ = stmt.raw_bind_parameter(2, time2.ux_ts());

        let mut rows = stmt.raw_query();
        get_row_val_f32(rows.next())
    }

}

#[inline]
#[rustfmt::skip]
fn update_vec(ds_row: &mut [f64; MAX_FEATURES], sum_wd_sin: &mut f64, sum_wd_cos: &mut f64, sensor_id: usize, value: f64, solar_rad: &mut f64) {
    match unsafe { Sensor::from_unchecked(sensor_id as u8) } {
        Sensor::HrOutside | Sensor::Pressure | Sensor::TempOutside | Sensor::WindSpeed | Sensor::DewPoint => {
            let idx = SENSOR_BASE_MAP[sensor_id];
            if ds_row[UPD_HELPER[idx][Sumarize::Avg as usize]].is_nan() { ds_row[UPD_HELPER[idx][Sumarize::Avg as usize]] = 0.; };  
            if ds_row[UPD_HELPER[idx][Sumarize::Max as usize]].is_nan() { ds_row[UPD_HELPER[idx][Sumarize::Max as usize]] = f64::MIN; }
            if ds_row[UPD_HELPER[idx][Sumarize::Min as usize]].is_nan() { ds_row[UPD_HELPER[idx][Sumarize::Min as usize]] = f64::MAX; } 
            ds_row[UPD_HELPER[idx][Sumarize::Max as usize]] = f64::max(ds_row[UPD_HELPER[idx][Sumarize::Max as usize]], value);
            ds_row[UPD_HELPER[idx][Sumarize::Min as usize]] = f64::min(ds_row[UPD_HELPER[idx][Sumarize::Min as usize]], value); 
            ds_row[UPD_HELPER[idx][Sumarize::Avg as usize]] += value;
        }
        Sensor::Rain => {
            if ds_row[Metric::SumRain as usize].is_nan() { ds_row[Metric::SumRain as usize] = 0.; }
            ds_row[Metric::SumRain as usize] += value;
        }
        Sensor::SolarRadiation => { *solar_rad += value; }
        Sensor::WindBearing => {
            *sum_wd_sin += f64::sin(value.to_radians());
            *sum_wd_cos += f64::cos(value.to_radians());
        }
        _ => (),  //são os sensores 7 e 9 e superiores, que não intereassam para o tempo
    };
}

#[inline]
fn incr_sensor_count (sensor_id: usize, ds_count: &mut [u16; MAX_SENSORS], ds_sensor: &mut [bool; MAX_SENSORS]) {
    if !ds_sensor[sensor_id] { ds_sensor[sensor_id] = true; }
    ds_count[sensor_id] += 1;
}

impl<'a> DBModelWeather<'a> for Persist {}
