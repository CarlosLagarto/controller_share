use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::data_structs::sensor::stat_metric::*;
use ctrl_lib::db::{db_error::*, db_sql_lite::*, SqlRow};
use ctrl_lib::services::weather:: rain_pred::data_structs::*;
use ctrl_lib::{log_warn, logger::*};
use ctrl_lib::services::weather::db_model::DBModelWeather;

use crate::integration::naive_bayes::code::data_structs::*;

pub trait DBModelWeatherNBMunch<'a>: DBModelWeather<'a> {

    // const ML_SELECT_RAW_DATA_DAY: &'a str = "select day_nr from ml_raw_data where day_nr=?;";

    const ML_SELECT_RAW_DATA: &'a str = "select day_nr,avg_temp,max_temp,min_temp,avg_press,max_press,min_press,avg_hr,max_hr,\
                                            min_hr,sum_prec,avg_ws,max_ws,min_ws,solarrad,avg_wd,temp_0,temp_6,temp_12,temp_18,\
                                            press_0,press_6,press_12,press_18,hr_0,hr_6,hr_12,hr_18,ws_0,ws_6,ws_12,ws_18,\
                                            wd_0,wd_6,wd_12,wd_18,avg_dwp,max_dwp,min_dwp,dwp_0,dwp_6,dwp_12,dwp_18,\
                                            press_dwp_ratio,sum_hr_gte_ratio,rain_class from ml_raw_data order by day_nr;";
    // const ML_SELECT_RAW_DATA_DAY_VEC: &'a str = "select day_nr,avg_temp,max_temp,min_temp,avg_press,max_press,min_press,avg_hr,max_hr,\
    //                                         min_hr,sum_prec,avg_ws,max_ws,min_ws,solarrad,avg_wd,temp_0,temp_6,temp_12,temp_18,\
    //                                         press_0,press_6,press_12,press_18,hr_0,hr_6,hr_12,hr_18,ws_0,ws_6,ws_12,ws_18,\
    //                                         wd_0,wd_6,wd_12,wd_18,avg_dwp,max_dwp,min_dwp,dwp_0,dwp_6,dwp_12,dwp_18,\
    //                                         press_dwp_ratio,sum_hr_gte_ratio,rain_class from ml_raw_data where day_nr=?;";

    // não atualizamos o sum_prec e rain_class, porque esta informação foi calculada para o dia, portanto deve atualizar o dia anterior
    // const ML_UPDATE_RAW_DATA: &'a str = "update ml_raw_data set avg_temp=?,max_temp=?,min_temp=?,avg_press=?,max_press=?,min_press=?,avg_hr=?,max_hr=?,\
    //                                         min_hr=?,sum_prec=?,avg_ws=?,max_ws=?,min_ws=?,solarrad=?,avg_wd=?,temp_0=?,temp_6=?,temp_12=?,temp_18=?,\
    //                                         press_0=?,press_6=?,press_12=?,press_18=?,hr_0=?,hr_6=?,hr_12=?,hr_18=?,ws_0=?,ws_6=?,ws_12=?,ws_18=?,\
    //                                         wd_0=?,wd_6=?,wd_12=?,wd_18=?,avg_dwp=?,max_dwp=?,min_dwp=?,dwp_0=?,dwp_6=?,dwp_12=?,dwp_18=?,\
    //                                         press_dwp_ratio=?,sum_hr_gte_ratio=?,rain_class=? where day_nr=?;";
    // const ML_UPDATE_RAW_DATA_FORECAST: &'a str = "update ml_raw_data set sum_prec=?,rain_class=? where day_nr=?;";

    // const ML_SELECT_RAW_DATA_FORECAST: &'a str = "select sum_prec, rain_class from ml_raw_data where day_nr=?;";
    // const ML_INSERT_RAW_DATA: &'a str = "insert into ml_raw_data(avg_temp,max_temp,min_temp,avg_press,max_press,min_press,avg_hr,max_hr,\
    //                                         min_hr,sum_prec,avg_ws,max_ws,min_ws,solarrad,avg_wd,temp_0,temp_6,temp_12,temp_18,\
    //                                         press_0,press_6,press_12,press_18,hr_0,hr_6,hr_12,hr_18,ws_0,ws_6,ws_12,ws_18,\
    //                                         wd_0,wd_6,wd_12,wd_18,avg_dwp,max_dwp,min_dwp,dwp_0,dwp_6,dwp_12,dwp_18,\
    //                                         press_dwp_ratio,sum_hr_gte_ratio,rain_class \
    //                                         VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,);";
    // const ML_SELECT_ML_MODEL: &'a str = "select model from ml_model_data where current_model_id=?;";
    const ML_INSERT_ML_MODEL: &'a str = "insert into ml_model_data (current_model_id, model, start_date) values (?,?,?)";
    const ML_UPDATE_ML_MODEL: &'a str = "update ml_model_data set end_date=? where current_model_id=?";
    const ML_UPDATE_ML_MODEL_TEST: &'a str = "update ml_model_data set model=? where current_model_id=?";

    // const ML_SELECT_ML_MODEL_ANALYSIS: &'a str = "select model, model_id, best_model from ml_explored_models where best_model=true;";
    // const ML_SELECT_ML_MODEL_BY_ID_ANALYSIS: &'a str = "select model, model_id, best_model from ml_explored_models where model_id=?;";
    const ML_INSERT_ML_MODEL_ANALYSIS: &'a str = "insert into ml_explored_models (model_id, model, best_model) values (?,?,?)";
    const ML_UPDATE_ML_MODEL_ANALYSIS: &'a str = "update ml_explored_models set best_model=? where model_id=?";
    const ML_DELETE_ML_MODEL_ANALYSIS: &'a str = "delete from ml_explored_models;";


    // aqui construimos o data set e misturamos responsabilidades,
    // mas já que estamos a fazer uma passagem pelo ds, calculamos coisas que são necessessárias mais á frente
    #[inline]
    fn get_ml_raw_data(&self, ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>, model: &mut Model) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::ML_SELECT_RAW_DATA).unwrap();
        let mut rows = stmt.raw_query();

        let mut aux_class: usize;
        let mut idx: usize = 0;
        while let Some(sql_row) = rows.next()? {
            let row: [f64; MAX_FEATURES] = into_vector(sql_row);
            ds.correlate_row(&row, idx);
            ds.push(row);
            aux_class = row[Metric::RainClass as usize].round() as usize;
            model.train_class_count[aux_class] += 1;
            ds.all_class_idxs[aux_class].push(idx);
            idx += 1;
        }

        Ok(())
    }

    // #[inline]
    // fn validate_and_upd_if_needed(&self, daily_vec: DSRow<MAX_FEATURES>) -> SimpleResult {
    //     let curr_day = daily_vec[Metric::DayNr as usize].round() as u16;

    //     //primeiro validamos
    //     let conn = &self.get_conn().conn;
    //     let mut stmt = conn.prepare_cached(Self::ML_SELECT_RAW_DATA_DAY).unwrap();
    //     _ = stmt.raw_bind_parameter(1, curr_day);
    //     let mut rows = stmt.raw_query();

    //     if let Some(_row) = rows.next()? {
    //         // se existir linha vamos fazer coisas
    //         // atualizamos a informação do dia - excepto o forecast sum_prec e rain_class
    //         let mut stmt = conn.prepare_cached(Self::ML_UPDATE_RAW_DATA).unwrap();
    //         bind_ml_raw_data(&mut stmt, &daily_vec);
    //         _ = self.exec_prep(&mut stmt);

    //         if let Some(_row) = rows.next()? {
    //             log_warn!("Existe mais do que uma linha para o mesmo dia na tabela ml_raw_data");
    //             // break;
    //         }
    //     } else {
    //         // vamos introduzir a linha
    //         // isto vai acontecer no próximo ano bisexto com 366 dias - só construi o data set para 365 dias
    //         // daí para a frente não deveria acontecer mais
    //         let mut stmt = conn.prepare_cached(Self::ML_INSERT_RAW_DATA).unwrap();
    //         bind_ml_raw_data(&mut stmt, &daily_vec);
    //         _ = self.exec_prep(&mut stmt);
    //     }
    //     Ok(())
    // }

    // #[inline]
    // fn upd_forecast_data(&self, mut ref_forecast_day: u16, sum_rain: f64, rain_class: f64) -> SimpleResult {
    //     let conn = &self.get_conn().conn;
    //     // e agora temos que atualizar o forecast com a informação da linha introduzida no dia anterior anterior
    //     if ref_forecast_day == 0 {
    //         // se estivermos no 0, quer dizer que vamos ter que ir para o 31 de dezembro, que será 365 ou 366,
    //         // dependendo se durante o funcionamento do programa já passamos por um ano bissexto ou não
    //         ref_forecast_day = 365; // simplificamos a coisa, e para este efeito não é relevante ter 365 ou 366 dias e referencias
    //     }
    //     // e agora atualizamos a informação do forecast
    //     let mut stmt = conn.prepare_cached(Self::ML_UPDATE_RAW_DATA_FORECAST).unwrap();
    //     _ = stmt.raw_bind_parameter(1, sum_rain);
    //     _ = stmt.raw_bind_parameter(2, rain_class);
    //     _ = stmt.raw_bind_parameter(3, ref_forecast_day);
    //     _ = self.exec_prep(&mut stmt);

    //     Ok(())
    // }

    // #[inline]
    // fn get_forecast_data(&self, ref_forecast_day: u16) -> Result<Option<(f64, f64)>, DBError> {
    //     let conn = &self.get_conn().conn;
    //     let mut stmt = conn.prepare_cached(Self::ML_SELECT_RAW_DATA_FORECAST).unwrap();
    //     _ = stmt.raw_bind_parameter(1, ref_forecast_day);
    //     let mut rows = stmt.raw_query();
    //     if let Some(row) = rows.next()? {
    //         let sum_rain = row.get(0).unwrap();
    //         let rain_class = row.get(1).unwrap();
    //         Ok(Some((sum_rain, rain_class)))
    //     } else {
    //         //pessegada porque não se obteve registos na query e deviamos
    //         log_warn!("Não foram encontrados nenhuns registos na tabela ml_raw_data");
    //         Ok(None)
    //     }
    // }

    #[inline]
    fn clean_explored_models(&self) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::ML_DELETE_ML_MODEL_ANALYSIS).unwrap();
        _ = self.exec_prep(&mut stmt);
        Ok(())
    }

    #[inline]
    fn update_explored_model(&self, model: u32, best_model: bool) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::ML_UPDATE_ML_MODEL_ANALYSIS).unwrap();
        _ = stmt.raw_bind_parameter(1, best_model);
        _ = stmt.raw_bind_parameter(2, model);
        _ = self.exec_prep(&mut stmt);
        Ok(())
    }

    #[inline]
    fn insert_explored_model(&self, model: u32, best_model: bool, model_str: String) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::ML_INSERT_ML_MODEL_ANALYSIS).unwrap();
        _ = stmt.raw_bind_parameter(1, model);
        _ = stmt.raw_bind_parameter(2, best_model);
        _ = stmt.raw_bind_parameter(3, model_str);
        _ = self.exec_prep(&mut stmt);
        Ok(())
    }

    // #[inline]
    // fn get_best_explored_model(&self) -> Result<Option<String>, DBError> {
    //     let conn = &self.get_conn().conn;
    //     let mut stmt = conn.prepare_cached(Self::ML_SELECT_ML_MODEL_ANALYSIS).unwrap();

    //     let mut rows = stmt.raw_query();
    //     if let Some(row) = rows.next()? {
    //         let model_str = row.get(0).unwrap();
    //         Ok(Some(model_str))
    //     } else {
    //         log_warn!("get_best_explored_model - Não foi encontrado nenhum modelo na tabela model_explored_models");
    //         Ok(None)
    //     }
    // }

    // #[inline]
    // fn get_explored_model_by_id(&self, model: u32) -> Result<Option<String>, DBError> {
    //     let conn = &self.get_conn().conn;
    //     let mut stmt = conn.prepare_cached(Self::ML_SELECT_ML_MODEL_BY_ID_ANALYSIS).unwrap();
    //     _ = stmt.raw_bind_parameter(1, model);

    //     let mut rows = stmt.raw_query();

    //     if let Some(row) = rows.next()? {
    //         let model_str = row.get(0).unwrap();
    //         Ok(Some(model_str))
    //     } else {
    //         warn!("get_explored_model_by_id - Não foi encontrado o modelo {} na tabela model_explored_models", model);
    //         Ok(None)
    //     }
    // }

    #[inline]
    fn save_model(&self, old_curr_model: u32, end_time: CtrlTime, new_curr_model: u32, model_str: String) -> SimpleResult {
        let aux = self.get_model(new_curr_model);
        if let Ok(Some(_m)) = aux {
            let conn = &self.get_conn().conn;
            // está-se a passar um id já existente para o novo model.
            // pode a acontecer nos testes.  Em produção não é suposto.
            let mut stmt = conn.prepare_cached(Self::ML_UPDATE_ML_MODEL_TEST).unwrap();
            _ = stmt.raw_bind_parameter(1, model_str);
            _ = stmt.raw_bind_parameter(2, new_curr_model);
            _ = self.exec_prep(&mut stmt);
        } else {
            let aux = self.get_model(old_curr_model);
            let conn = &self.get_conn().conn;
            if let Ok(Some(_m)) = aux {
                // validamos se o old model existe antes de o atualizarmos
                let mut stmt = conn.prepare_cached(Self::ML_UPDATE_ML_MODEL).unwrap();
                _ = stmt.raw_bind_parameter(1, end_time.ux_ts());
                _ = stmt.raw_bind_parameter(2, old_curr_model);
                _ = self.exec_prep(&mut stmt);
            } else {
                log_warn!("Atualização de um modelo não inexistente.")
            }
            // e finalmente inserimos o novo modelo
            let mut stmt = conn.prepare_cached(Self::ML_INSERT_ML_MODEL).unwrap();
            _ = stmt.raw_bind_parameter(1, new_curr_model);
            _ = stmt.raw_bind_parameter(2, model_str);
            _ = stmt.raw_bind_parameter(3, end_time.ux_ts());
            _ = self.exec_prep(&mut stmt);
        }
        Ok(())
    }

    // #[inline]
    // fn get_day_ml_rec(&self, day_ref: CtrlTime) -> Result<Option<DSRow<MAX_FEATURES>>, DBError> {
    //     //primeiro validamos
    //     let conn = &self.get_conn().conn;
    //     let mut stmt = conn.prepare_cached(Self::ML_SELECT_RAW_DATA_DAY_VEC).unwrap();
    //     _ = stmt.raw_bind_parameter(1, day_ref.year_day_number_e());
    //     let mut rows = stmt.raw_query();

    //     let mut nr: [f64; MAX_FEATURES] = [f64::NAN; MAX_FEATURES];
    //     let mut counter = 0;

    //     while let Some(row) = rows.next()? {
    //         nr = into_vector(row);
    //         counter += 1;
    //         if counter == 2 {
    //             log_warn!("Existe mais do que uma linha para o mesmo dia na tabela ml_raw_data, e só devia ser uma linha.");
    //             break;
    //         }
    //     }
    //     if counter == 0 {
    //         Ok(None)
    //     } else {
    //         Ok(Some(nr))
    //     }
    // }

}

// #[allow(dead_code)]
// // é chamada do validate e update, que por sua vez só será utilizado para o processamento manual ou testes
// fn bind_ml_raw_data(stmt: &mut CachedStatement, daily_vec: &[f64; MAX_FEATURES]) {
//     _ = stmt.raw_bind_parameter(1, daily_vec[Metric::AvgTemp as usize]);
//     _ = stmt.raw_bind_parameter(2, daily_vec[Metric::MaxTemp as usize]);
//     _ = stmt.raw_bind_parameter(3, daily_vec[Metric::MinTemp as usize]);
//     _ = stmt.raw_bind_parameter(4, daily_vec[Metric::AvgPressure as usize]);
//     _ = stmt.raw_bind_parameter(5, daily_vec[Metric::MaxPressure as usize]);
//     _ = stmt.raw_bind_parameter(6, daily_vec[Metric::MinPressure as usize]);
//     _ = stmt.raw_bind_parameter(7, daily_vec[Metric::AvgHumidity as usize]);
//     _ = stmt.raw_bind_parameter(8, daily_vec[Metric::MaxHumidity as usize]);
//     _ = stmt.raw_bind_parameter(9, daily_vec[Metric::MinHumidity as usize]);
//     // _ = stmt.raw_bind_parameter(10, daily_vec[StatMetric::SumRain as usize]);
//     _ = stmt.raw_bind_parameter(10, daily_vec[Metric::AvgWindSpeed as usize]);
//     _ = stmt.raw_bind_parameter(11, daily_vec[Metric::MaxWindSpeed as usize]);
//     _ = stmt.raw_bind_parameter(12, daily_vec[Metric::MinWindSpeed as usize]);
//     _ = stmt.raw_bind_parameter(13, daily_vec[Metric::SolarRadiation as usize]);
//     _ = stmt.raw_bind_parameter(14, daily_vec[Metric::AvgWindDirection as usize]);
//     _ = stmt.raw_bind_parameter(15, daily_vec[Metric::TempAt0 as usize]);
//     _ = stmt.raw_bind_parameter(16, daily_vec[Metric::TempAt6 as usize]);
//     _ = stmt.raw_bind_parameter(17, daily_vec[Metric::TempAt12 as usize]);
//     _ = stmt.raw_bind_parameter(18, daily_vec[Metric::TempAt18 as usize]);
//     _ = stmt.raw_bind_parameter(19, daily_vec[Metric::PressAt0 as usize]);
//     _ = stmt.raw_bind_parameter(20, daily_vec[Metric::PressAt6 as usize]);
//     _ = stmt.raw_bind_parameter(21, daily_vec[Metric::PressAt12 as usize]);
//     _ = stmt.raw_bind_parameter(22, daily_vec[Metric::PressAt18 as usize]);
//     _ = stmt.raw_bind_parameter(23, daily_vec[Metric::HrAt0 as usize]);
//     _ = stmt.raw_bind_parameter(24, daily_vec[Metric::HrAt6 as usize]);
//     _ = stmt.raw_bind_parameter(25, daily_vec[Metric::HrAt12 as usize]);
//     _ = stmt.raw_bind_parameter(26, daily_vec[Metric::HrAt18 as usize]);
//     _ = stmt.raw_bind_parameter(27, daily_vec[Metric::WsAt0 as usize]);
//     _ = stmt.raw_bind_parameter(28, daily_vec[Metric::WsAt6 as usize]);
//     _ = stmt.raw_bind_parameter(29, daily_vec[Metric::WsAt12 as usize]);
//     _ = stmt.raw_bind_parameter(30, daily_vec[Metric::WsAt18 as usize]);
//     _ = stmt.raw_bind_parameter(31, daily_vec[Metric::WdAt0 as usize]);
//     _ = stmt.raw_bind_parameter(32, daily_vec[Metric::WdAt6 as usize]);
//     _ = stmt.raw_bind_parameter(33, daily_vec[Metric::WdAt12 as usize]);
//     _ = stmt.raw_bind_parameter(34, daily_vec[Metric::WdAt18 as usize]);
//     _ = stmt.raw_bind_parameter(35, daily_vec[Metric::AvgDwp as usize]);
//     _ = stmt.raw_bind_parameter(36, daily_vec[Metric::MaxDwp as usize]);
//     _ = stmt.raw_bind_parameter(37, daily_vec[Metric::MinDwp as usize]);
//     _ = stmt.raw_bind_parameter(38, daily_vec[Metric::DwpAt0 as usize]);
//     _ = stmt.raw_bind_parameter(39, daily_vec[Metric::DwpAt6 as usize]);
//     _ = stmt.raw_bind_parameter(40, daily_vec[Metric::DwpAt12 as usize]);
//     _ = stmt.raw_bind_parameter(41, daily_vec[Metric::DwpAt18 as usize]);
//     _ = stmt.raw_bind_parameter(42, daily_vec[Metric::PressureDwpRatio as usize]);
//     _ = stmt.raw_bind_parameter(43, daily_vec[Metric::HumidityGtERatio as usize]);
//     // _ = stmt.raw_bind_parameter(45, daily_vec[StatMetric::RainClass as usize]);
//     _ = stmt.raw_bind_parameter(44, daily_vec[Metric::DayNr as usize]);
// }

fn into_vector(row: &SqlRow) -> [f64; MAX_FEATURES] {
    let mut nr: [f64; MAX_FEATURES] = [f64::NAN; MAX_FEATURES];
    nr[Metric::DayNr as usize] = row.get(0).unwrap();
    nr[Metric::AvgTemp as usize] = row.get(1).unwrap();
    nr[Metric::MaxTemp as usize] = row.get(2).unwrap();
    nr[Metric::MinTemp as usize] = row.get(3).unwrap();
    nr[Metric::AvgPressure as usize] = row.get(4).unwrap();
    nr[Metric::MaxPressure as usize] = row.get(5).unwrap();
    nr[Metric::MinPressure as usize] = row.get(6).unwrap();
    nr[Metric::AvgHumidity as usize] = row.get(7).unwrap();
    nr[Metric::MaxHumidity as usize] = row.get(8).unwrap();
    nr[Metric::MinHumidity as usize] = row.get(9).unwrap();
    nr[Metric::SumRain as usize] = row.get(10).unwrap();
    nr[Metric::AvgWindSpeed as usize] = row.get(11).unwrap();
    nr[Metric::MaxWindSpeed as usize] = row.get(12).unwrap();
    nr[Metric::MinWindSpeed as usize] = row.get(13).unwrap();
    nr[Metric::SolarRadiation as usize] = row.get(14).unwrap();
    nr[Metric::AvgWindDirection as usize] = row.get(15).unwrap();
    nr[Metric::TempAt0 as usize] = row.get(16).unwrap();
    nr[Metric::TempAt6 as usize] = row.get(17).unwrap();
    nr[Metric::TempAt12 as usize] = row.get(18).unwrap();
    nr[Metric::TempAt18 as usize] = row.get(19).unwrap();
    nr[Metric::PressAt0 as usize] = row.get(20).unwrap();
    nr[Metric::PressAt6 as usize] = row.get(21).unwrap();
    nr[Metric::PressAt12 as usize] = row.get(22).unwrap();
    nr[Metric::PressAt18 as usize] = row.get(23).unwrap();
    nr[Metric::HrAt0 as usize] = row.get(24).unwrap();
    nr[Metric::HrAt6 as usize] = row.get(25).unwrap();
    nr[Metric::HrAt12 as usize] = row.get(26).unwrap();
    nr[Metric::HrAt18 as usize] = row.get(27).unwrap();
    nr[Metric::WsAt0 as usize] = row.get(28).unwrap();
    nr[Metric::WsAt6 as usize] = row.get(29).unwrap();
    nr[Metric::WsAt12 as usize] = row.get(30).unwrap();
    nr[Metric::WsAt18 as usize] = row.get(31).unwrap();
    nr[Metric::WdAt0 as usize] = row.get(32).unwrap();
    nr[Metric::WdAt6 as usize] = row.get(33).unwrap();
    nr[Metric::WdAt12 as usize] = row.get(34).unwrap();
    nr[Metric::WdAt18 as usize] = row.get(35).unwrap();
    nr[Metric::AvgDwp as usize] = row.get(36).unwrap();
    nr[Metric::MaxDwp as usize] = row.get(37).unwrap();
    nr[Metric::MinDwp as usize] = row.get(38).unwrap();
    nr[Metric::DwpAt0 as usize] = row.get(39).unwrap();
    nr[Metric::DwpAt6 as usize] = row.get(40).unwrap();
    nr[Metric::DwpAt12 as usize] = row.get(41).unwrap();
    nr[Metric::DwpAt18 as usize] = row.get(42).unwrap();
    nr[Metric::PressureDwpRatio as usize] = row.get(43).unwrap();
    nr[Metric::HumidityGtERatio as usize] = row.get(44).unwrap();
    nr[Metric::RainClass as usize] = row.get::<usize, f64>(45).unwrap();
    nr
}

impl<'a> DBModelWeatherNBMunch<'a> for Persist {}
