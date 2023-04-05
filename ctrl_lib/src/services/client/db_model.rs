use crate::app_time::ctrl_time::*;
use crate::db::{*, db_error::*, db_sql_lite::*};

#[derive(Default)]
pub struct DailyWeatherFromDB {
    pub et: f32,
    pub rain_probability: f32,
    pub rain_class_forecast: f32,
}

impl From<&SqlRow<'_>> for DailyWeatherFromDB {
    #[inline]
    fn from(sql_row: &SqlRow) -> DailyWeatherFromDB {
        DailyWeatherFromDB {
            et: sql_row.get(0).unwrap(),
            rain_probability: sql_row.get(1).unwrap(),
            rain_class_forecast: sql_row.get(2).unwrap(),
        }
    }
}

pub trait DBModelClient<'a>: DB {
    const GET_DAILY_DATA: &'a str = "select et, rain_probability, rain_class_forecast from\
                    (select timestamp,\
                    max(CASE WHEN id_metric=8 THEN value END) AS et,\
                    max(CASE WHEN id_metric=47 THEN value END) AS rain_probability,\
                    max(CASE WHEN id_metric=46 THEN value END) AS rain_class_forecast \
                    from sensor_daily_data \
                    where (id_metric=8 or id_metric=46 or id_metric=47) and timestamp=? group by timestamp);";
    //Sabemos que a chuva é o sensor 0
    const GET_RAIN_BETWEEN: &'a str = "SELECT coalesce(sum(value),0)as total from sensor_data where timestamp>=? and timestamp<=? and id_sensor=0;"; 

    #[inline]
    fn get_daily_data(&self, time: CtrlTime) -> Result<DailyWeatherFromDB, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_DAILY_DATA).unwrap();
        _ = stmt.raw_bind_parameter(1, time.ux_ts());

        let mut rows = stmt.raw_query();
        let mut data = DailyWeatherFromDB::default();
        if let Some(row) = rows.next()? {
            data = row.into();
        }
        Ok(data)
    }

    // Assume que devolve sempre uma linha, com uma coluna apenas
    #[inline]
    fn get_daily_rain(&self, time: CtrlTime) -> Option<f32> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_RAIN_BETWEEN).unwrap();
        _ = stmt.raw_bind_parameter(1, time.sod_ux_e().ux_ts());
        _ = stmt.raw_bind_parameter(2, time.eod_ux_e().ux_ts());

        let mut rows = stmt.raw_query();
        get_row_val_f32(rows.next())
    }

    // Assume que devolve sempre uma linha, com uma coluna apenas
    #[inline]
    fn get_week_acc_rain(&self, time: CtrlTime) -> Option<f32> {
        let conn = &self.get_conn().conn;
        let start = time.sub_days(time.week_day_e() as u64).sod_ux_e();
        let end = start.add_days(6).eod_ux_e();
        let mut stmt = conn.prepare_cached(Self::GET_RAIN_BETWEEN).unwrap();
        _ = stmt.raw_bind_parameter(1, start.ux_ts());
        _ = stmt.raw_bind_parameter(2, end.ux_ts());

        let mut rows = stmt.raw_query();
        get_row_val_f32(rows.next())
    }

}

impl<'a> DBModelClient<'a> for Persist {}
