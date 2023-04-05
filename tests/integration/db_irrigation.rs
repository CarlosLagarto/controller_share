#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
use core::slice;
use std::cell::RefCell;
use std::io::{BufWriter, Write};
use std::rc::Rc;
use std::{sync::Arc, time::*};

use ctrl_lib::app_time::{ctrl_time::*, tm::*};
use ctrl_lib::config::{geo_pos::*, wtr_cfg::*, Module};
use ctrl_lib::data_structs::client::sync_op::*;
use ctrl_lib::data_structs::rega::{mode::*, watering_status::*, wizard_info::*};
use ctrl_lib::data_structs::sensor::{daily_value::*, snsor::*, stat_metric::*, *};
use ctrl_lib::db::{db_error::*, db_sql_lite::*, *};
use ctrl_lib::services::irrigation::states::starting::*;
use ctrl_lib::services::irrigation::{
    cycle::*, cycle_run::*, cycle_type::*, db_model::*, sector::*, sector_run::*, wtr_engine::*, wzrd_algorithms::*,
};
use ctrl_lib::services::weather::db_model::DBModelWeather;
use ctrl_lib::utils::*;
use ctrl_lib::{logger::*, ArrayVec};
use ctrl_prelude::{domain_types::*, globals::*};

use crate::integration::water_service::*;

const ITERS: u64 = 100;

pub trait DBModelRegaTest<'a>: DBModelRega<'a> {
    fn delete_watered_sector(&self, cycle_id: CYCLE_ID, run: u32, sector_id: DEVICE_ID) -> SimpleResult {
        let sql: &str = "delete from watered_sector where id_ciclo = ? and current_run = ? and id_sector = ?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();

        let mut _res = stmt.raw_bind_parameter(1, cycle_id);
        _res = stmt.raw_bind_parameter(2, run);
        _res = stmt.raw_bind_parameter(3, sector_id);

        self.exec_prep(&mut stmt)
    }

    fn delete_watered_cycle(&self, cycle_id: CYCLE_ID, run: u32) -> SimpleResult {
        let sql: &str = "delete from watered_cycle where id_ciclo=? and current_run=?";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();

        let mut _res = stmt.raw_bind_parameter(1, cycle_id);
        _res = stmt.raw_bind_parameter(2, run);

        self.exec_prep(&mut stmt)
    }

    fn ins_daily_data_batch_aux(&self, rows: &[SensorValue]) -> SimpleResult {
        let sql: &'a str = "INSERT INTO sensor_daily_data(id_metric,timestamp,value)VALUES(?,?,?);";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql).unwrap();
        let mut res: SimpleResult = Ok(());

        for row in rows {
            _ = stmt.raw_bind_parameter(1, row.id as u8);
            _ = stmt.raw_bind_parameter(2, row.timestamp.ux_ts());
            _ = stmt.raw_bind_parameter(3, row.value);
            res = self.exec_prep(&mut stmt);
        }
        res
    }
}

impl<'a> DBModelRegaTest<'a> for Persist {}

#[test]
fn db_get_config_sectors_sql_stmt_is_ok() {
    //oracle type testing.  Sectors config info is a manual entry in the database
    //we know which entries are there so testing exercise it's just to assure that we have access to the database
    //and that the sql statement is ok
    let db = Persist::new();
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    let result = db.get_cfg_secs(&mut list);
    assert!(result.is_ok());
}

#[test]
fn db_get_config_sectors_sql_stmt_is_ok_time() {
    let mut total: u64 = 0;
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    let db = Persist::new();
    for _i in 0..ITERS {
        list.clear();
        let t = Instant::now();
        let _result = db.get_cfg_secs(&mut list);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_config_sectors_sql_stmt_is_ok_time: {}", elapsed_dyn((total / ITERS) as u64));
}

#[test]
fn db_get_rain_since_ref_empty_db() {
    let db = Persist::new();

    let date_ref = CtrlTime::from_utc_parts(2022, 08, 01, 0, 0, 0);
    _ = db.del_all_sensor_data();

    let res = db.get_rain_between(date_ref, date_ref.add_days(1));
    if let Some(val) = res{
        assert!((val-0.).abs() <= 0.00001);
    }else{
        assert!(false);
    }
}

#[test]
fn db_get_rain_since_ref_empty_db_time() {
    let mut total: u64 = 0;
    let db = Persist::new();
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.get_rain_between(CtrlTime(0), CtrlTime(CtrlTime::MAX));
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_rain_since_ref_empty_db_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_find_schedule_cycle() {
    let db = Persist::new();
    let result = db.find_cycle(CtrlTime(0), CtrlTime(0));
    assert!(result.is_ok());
}

#[test]
fn db_find_schedule_cycle_time() {
    let db = Persist::new();
    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.find_cycle(CtrlTime(0), CtrlTime(0));
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_find_schedule_cycle_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_get_all_schedule_cycles() {
    let db = Persist::new();
    let mut list: CycleList = ArrayVec::<Cycle, MAX_CYCLES>::new();
    let result = db.get_all_cycles(&mut list);
    assert!(result.is_ok());
}

#[test]
fn db_get_all_schedule_cycles_time() {
    let db = Persist::new();
    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let mut list: CycleList = ArrayVec::<Cycle, MAX_CYCLES>::new();
        let t = Instant::now();
        let _result = db.get_all_cycles(&mut list);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_get_all_schedule_cycles_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_update_sector() {
    let db = Persist::new();
    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    let rs = &sectors[0];
    let result = db.upd_sec(&rs);
    assert!(result.is_ok());
}

#[test]
fn db_update_sector_time() {
    let db = Persist::new();
    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    let rs = &sectors[0];

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.upd_sec(&rs);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_update_sector_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_update_scheduled_cycle_server() {
    let db = Persist::new();
    let mut c: Cycle = Cycle::default();
    c.run.cycle_id = 1;
    // c.sim = 0;
    let result = db.upd_cycle_srvr(&c);
    assert!(result.is_ok());
}

#[test]
fn db_update_scheduled_cycle_server_time() {
    let db = Persist::new();
    let mut c: Cycle = Cycle::default();
    c.run.cycle_id = 1;

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.upd_cycle_srvr(&c);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_update_scheduled_cycle_server_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_update_watered_cycle() {
    let db = Persist::new();
    let c = CycleRun { cycle_id: 1, ..Default::default() };
    let result = db.upd_cycle_run(&c);
    assert!(result.is_ok());
}

#[test]
fn db_update_watered_cycle_time() {
    let db = Persist::new();
    let c = CycleRun { cycle_id: 1, ..Default::default() };

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.upd_cycle_run(&c);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_update_watered_cycle_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_update_watered_sector() {
    let db = Persist::new();
    let c = SectorRun { cycle_id: 1, sec_id: 0, curr_run: 0, ..Default::default() };
    let result = db.upd_sec_run(&c);
    assert!(result.is_ok());
}

#[test]
fn db_update_watered_sector_time() {
    let db = Persist::new();
    let c = SectorRun { cycle_id: 1, sec_id: 0, curr_run: 0, ..Default::default() };

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.upd_sec_run(&c);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_update_watered_sector_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_update_sectors_batch() {
    let db = Persist::new();
    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    let mut secs: Vec<Sector> = Vec::with_capacity(6);
    let s = sectors[0].clone();
    secs.push(s);
    let result = db.upd_secs_batch(&secs);
    assert!(result.is_ok());
}

#[test]
fn db_update_sectors_batch_time() {
    let db = Persist::new();
    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    let mut secs: Vec<Sector> = Vec::with_capacity(6);

    for j in 0..6 {
        secs.push(sectors[j].clone());

        let mut total: u64 = 0;
        for _i in 0..ITERS {
            let t = Instant::now();
            let _result = db.upd_secs_batch(&secs);
            total += (Instant::now() - t).as_nanos() as u64;
        }
        println!("db_update_sectors_batch_time {} sector: {}", j, elapsed_dyn(total / ITERS));
    }
}

#[test]
fn db_delete_scheduled_cycle_by_id() {
    let db = Persist::new();
    let result = db.del_cycle_by_id(1);
    assert!(result.is_ok());
}

#[test]
fn db_delete_scheduled_cycle_by_id_time() {
    let db = Persist::new();
    //tenho que introduzir algo antes
    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.del_cycle_by_id(1);
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_delete_scheduled_cycle_by_id_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_insert_scheduled_cycle() {
    let db = Persist::new();
    let mut c: Cycle = Cycle::default();
    c.run.cycle_id = 1;
    // c.sim = 0;
    let _result = db.ins_cycle(&mut c);
    let _result = db.del_cycle_by_id(1);
    assert!(_result.is_ok());
}

#[test]
fn db_insert_scheduled_cycle_time() {
    let db = Persist::new();
    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let mut c: Cycle = Cycle::default();
        c.run.cycle_id = 1;

        let t = Instant::now();
        let _result = db.ins_cycle(&mut c);
        total += (Instant::now() - t).as_nanos() as u64;
        let _result = db.del_cycle_by_id(1);
    }
    println!("db_insert_scheduled_cycle_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_insert_watered_cycle() {
    let db = Persist::new();
    let mut c: Cycle = Cycle::default();
    c.run.cycle_id = 1;
    c.run.run_id = 1;

    let _result = db.delete_watered_cycle(1, 1);

    let result = db.ins_cycle_run(&c.run);
    println!("{:?}", result);
    assert!(result.is_ok());

    let _result = db.delete_watered_cycle(1, 1);
}

#[test]
fn db_insert_watered_cycle_time() {
    let db = Persist::new();
    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let mut c: Cycle = Cycle::default();
        c.run.cycle_id = 1;
        c.run.run_id = 1;

        let t = Instant::now();
        let _result = db.ins_cycle_run(&c.run);
        total += (Instant::now() - t).as_nanos() as u64;
        let _result = db.delete_watered_cycle(1, 1);
    }
    println!("db_insert_watered_cycle_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_recover_inconsistent_schedule_cycles_new() {
    let db = Persist::new();
    let result = db.recover_inconsistent_cycles();
    assert!(result.is_ok());
}

#[test]
fn db_recover_inconsistent_schedule_cycles_new_time() {
    let db = Persist::new();

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.recover_inconsistent_cycles();
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_recover_inconsistent_schedule_cycles_new_time: {}", elapsed_dyn(total / ITERS));
}

#[test]
fn db_recover_watered_cycle_new() {
    let db = Persist::new();
    let result = db.recover_cycle_run();
    assert!(result.is_ok());
}

#[test]
fn db_recover_watered_cycle_new_time() {
    let db = Persist::new();

    let mut total: u64 = 0;
    for _i in 0..ITERS {
        let t = Instant::now();
        let _result = db.recover_cycle_run();
        total += (Instant::now() - t).as_nanos() as u64;
    }
    println!("db_recover_watered_cycle_new_time: {}", elapsed_dyn(total / ITERS));
}

// testa o fresh start
#[test]
fn wtr_establish_initial_condition_fresh_start() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let last_save = CtrlTime(0);
    let (db, _start_up) = prepare_common(start_time, Mode::Standard, last_save);

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(last_save.ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    for sec in list.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
    }
    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    establish_initial_conditions(&mut wtr_cfg, &mut list);

    for sec in list.iter() {
        assert_approx_eq!(sec.deficit, 0.);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }
}

// testa o rearranque - smoke test com tempo de paragem inferior a 1 dia
#[test]
fn wtr_establish_initial_condition_stop_start_less_one_day() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let last_save = CtrlTime(0);
    let (db, _start_up) = prepare_common(start_time, Mode::Standard, last_save);

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(last_save.ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    for sec in list.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
    }
    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    establish_initial_conditions(&mut wtr_cfg, &mut list);

    for sec in list.iter() {
        assert_approx_eq!(sec.deficit, 0.);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }

    // data for restart
    let new_start_time = CtrlTime::from_utc_parts(2022, 01, 22, 2, 0, 0);
    wtr_cfg.last_saved = new_start_time.sub_secs_f32(3600.); // parou uma hora antes
    wtr_cfg.fresh_start = 1;

    // e volta a arrancar
    wtr_cfg.live_since = new_start_time;

    establish_initial_conditions(&mut wtr_cfg, &mut list);

    // deverá continuar tudo a zero porque foi o rearranque foi < 1 dia
    for sec in list.iter() {
        assert_approx_eq!(sec.deficit, 0.);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }
}

// testa o rearranque - com > 1 dia de paragem - 1,x dias de paragem
#[test]
fn wtr_establish_initial_condition_stop_start_one_day_and_something() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let last_save = CtrlTime(0);
    let (db, _start_up) = prepare_common(start_time, Mode::Standard, last_save);

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(last_save.ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    for sec in list.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
    }
    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    establish_initial_conditions(&mut wtr_cfg, &mut list);

    for sec in list.iter() {
        assert_approx_eq!(sec.deficit, 0.);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }

    let nr_dias_parado = 1;
    // data for restart
    test_with_stoped_days(start_time, nr_dias_parado, &mut wtr_cfg, &mut list);
}

fn test_with_stoped_days(start_time: CtrlTime, nr_dias_parado: u64, mut wtr_cfg: &mut WtrCfg, list: &mut SecList) {
    // só conta como 1 dia a partir do sod day do start anterior e mais um dia , x em cima
    let new_start_time = start_time.sod_ux_e().add_days(nr_dias_parado) + GIGA_U;
    //somo 1 segundo pelo que se as contas estão certas, dá 1 dia inteiro e uns pozitos
    wtr_cfg.last_saved = start_time.add_secs_f32(3600.);
    // parou uma hora depois de arrancar
    wtr_cfg.fresh_start = 1;
    // e volta a arrancar
    wtr_cfg.live_since = new_start_time;
    println!("last stop: {}", wtr_cfg.last_stop.sod_ux_e().as_rfc3339_str_e());
    println!("live since: {}", wtr_cfg.live_since.sod_ux_e().as_rfc3339_str_e());
    println!("last saved: {}", wtr_cfg.last_saved.as_rfc3339_str_e());
    establish_initial_conditions(&mut wtr_cfg, list);
    // deverá dar um defict de o et standard de 1 dia + o percolation de um dia, porque não há weather nesse periodo (quando para a maquina para tudo)
    // e aqui está uma suposta vantagem de ter processos separados, mas voltamos sempre ao mesmo.  ISto quando para, é porque parou o pc, pelo que para sempre tdo
    // a excepção seria a manutenção só da rega, desabilitando os setores ou pondo em modo manual, mas vou deixar isso para outras nupcias
    let et = nr_dias_parado as f32 * wtr_cfg.wizard_info.daily_tgt_grass_et;
    for sec in list.iter() {
        let percolation = nr_dias_parado as f32 * (sec.percolation * 24.);
        let calc_defict = f32::clamp(percolation + et, 0., GRASS_ROOT_LENGTH);
        assert_approx_eq!(sec.deficit, calc_defict);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }
}

// testa o rearranque - com > 1 dia de paragem
#[test]
fn wtr_establish_initial_condition_stop_start_one_day_plus() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let last_save = CtrlTime(0);
    let (db, _start_up) = prepare_common(start_time, Mode::Standard, last_save);

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(last_save.ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    for sec in list.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
    }
    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    establish_initial_conditions(&mut wtr_cfg, &mut list);

    for sec in list.iter() {
        assert_approx_eq!(sec.deficit, 0.);
        assert!(sec.last_change == CtrlTime(0));
        assert!(sec.last_watered_in == CtrlTime(0));
    }

    // data for restart
    for nr_dias_parado in 1..101 {
        test_with_stoped_days(start_time, nr_dias_parado, &mut wtr_cfg, &mut list);

        for sec in list.iter_mut() {
            sec.deficit = 0.;
            sec.last_change = CtrlTime(0);
            sec.last_watered_in = CtrlTime(0);
        }
    }
}

// TODO - agora o que falta testar acima, é o rearranque  com algum deficit registaod e/ou com chuva registada pelo meio
// ,as os casos de teste acima já comprovam a funcionalidade core.  O resto é suposto funcionar.....famous last words :-)

// cenário base - happy path com arranque pela primeira vez
// não há dados de sensores, pelo que dará o valor por defeito na duração e zero no deficit
pub const ORACLE_DEFAULT_ET: f32 = 3.571428571; //está na bd, mas é para aproveotar e detetar algum bug se for o caso
pub const ORACLE_ET: f32 = 3.;
use assert_approx_eq::*;

use super::db_weather::DBModelWeatherTest;

#[test]
fn wtr_wizard_strategy_base_001() {
    let db = Persist::new();
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();
    let sec = &mut list[0];

    sec.deficit = 0.;
    sec.last_watered_in = CtrlTime(0);

    let mut oracle_dur = f32::ceil(ORACLE_DEFAULT_ET / sec.debit);
    // as regras aplicadas pelo wizard estabelecem que <= 1 os minutos serão zero
    if oracle_dur <= 1.{
        oracle_dur = 0.;
    }

    let time_ref = CtrlTime::from_utc_parts(1970, 01, 01, 0, 0, 0);
    let mut wtr_cfg = WtrCfg::new(db.clone(), time_ref);
    if wtr_cfg.live_since > time_ref {
        wtr_cfg.live_since = time_ref;
    }

    let minutes: f32;
    let deficit: f32;

    (minutes, deficit) = dur_wzrd_strategy(&db, &sec, &wtr_cfg, time_ref);
    println!("Cenário base - dia {} - deficit a {} -> Minutos = {}", time_ref.as_rfc3339_str_e(), &deficit, &minutes);
    assert_approx_eq!(oracle_dur, minutes, 0.001);
    assert_approx_eq!(ORACLE_DEFAULT_ET, deficit, 0.001);
}

pub struct Values {
    pub today_rain: f32,
    pub temp: f32,
    pub wind_b: f32,
    pub wind_i: f32,
    pub hr: f32,
    pub press: f32,
}

pub fn ins_sensor_data(time: CtrlTime, values: Values, db: &Persist) {
    let mut sensor_data_buf: ArrayVec<SensorValue, MAX_SENSORS> = ArrayVec::<SensorValue, MAX_SENSORS>::new();
    sensor_data_buf.push(SensorValue::new(Sensor::Rain as u8, time, values.today_rain));
    sensor_data_buf.push(SensorValue::new(Sensor::TempOutside as u8, time, values.temp));
    sensor_data_buf.push(SensorValue::new(Sensor::WindBearing as u8, time, values.wind_b));
    sensor_data_buf.push(SensorValue::new(Sensor::WindSpeed as u8, time, values.wind_i));
    sensor_data_buf.push(SensorValue::new(Sensor::HrOutside as u8, time, values.hr));
    sensor_data_buf.push(SensorValue::new(Sensor::Pressure as u8, time, values.press));

    _ = db.ins_sensor_data_batch(&sensor_data_buf);
}

// cenário "normal" durante um dois dias (com et e com rain)
#[test]
fn wtr_wizard_strategy_base_002() {
    let start_time = CtrlTime::from_utc_parts(1970, 02, 01, 1, 0, 0);

    let db = Persist::new();

    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    // live since ever - maior do que zero por causa de se utilizar o zero para validar que nunca correu
    wtr_cfg.live_since = CtrlTime::from_utc_parts(1970, 02, 01, 0, 0, 1);

    // criação das condições
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();

    let nr_of_days = 2; // nr de dias que vamos testar

    // criamos os dados
    create_data(start_time.sub_days(1).sod_ux_e(), &db, nr_of_days + 1, 1. / 240.); //a ver se crio os dados na data certa

    // ir buscar setores e estabelecer condições
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    let sec = &mut list[0];
    // o setor correu no dia anterior, ou seja, já correu
    sec.deficit = 0.;
    sec.last_watered_in = start_time;
    // simula 1 mm rain/ dia
    simulate_machine_run(nr_of_days, start_time, sec, &db, &wtr_cfg, true, 1.);
}

#[rustfmt::skip]
fn simulate_machine_run( nr_of_days: u64, start_time: CtrlTime, sec: &mut Sector, db: &Persist, wtr_cfg: &WtrCfg, et_in_db: bool, oracle_rain_val: f32, ) {
    let mut minutes: f32;
    let mut deficit: f32;
    let mut oracle_dur: f32;
    let mut oracle_deficit_new: f32;
    let mut wtr_time_ref: CtrlTime = start_time;
    let mut d = 0;
    while d < nr_of_days {
        // simulamos o water stop
        sec.last_watered_in = wtr_time_ref;
        wtr_time_ref = start_time.add_days(d + 1);

        // construimos os dados do oraculo a ver se o pgm está ok
        // percolation desde a ultima vez que regou
        // valor de ref para a validação - é o que colocamos na BD para simular o "medido" para ser diferente do que está parametrizado no setor
        let oracle_percolation = nano_to_min(wtr_time_ref.0 - sec.last_watered_in.0) * (sec.percolation / 60.);
        // et de teste
        if et_in_db {
            oracle_deficit_new = ORACLE_ET + oracle_percolation;
        } else {
            oracle_deficit_new = ORACLE_DEFAULT_ET + oracle_percolation;
        }

        oracle_deficit_new -= oracle_rain_val;
        oracle_deficit_new += sec.deficit;

        // temos que simular o stress control por causa da chuva
        sec.deficit -= oracle_rain_val; //o stress control trabalha de 6 em 6 minutos, mas aqui pomos o valor diário passado no parametro
        oracle_dur = f32::ceil(oracle_deficit_new / sec.debit).min(30.).max(0.);

        if oracle_dur <= 1. {
            oracle_dur = 0.;
        }
        oracle_deficit_new = oracle_deficit_new.clamp(-1., 150.);

        // e agora ir buscar o valor do pgm a ver se bate certo
        (minutes, deficit) = dur_wzrd_strategy(&db, &sec, &wtr_cfg, wtr_time_ref);

        // validamos a info
        println!("Cenário base - dia {} - deficit a {} -> minutos = {}", wtr_time_ref.as_rfc3339_str_e(), &deficit, &minutes);
        assert_approx_eq!(oracle_dur, minutes, 0.001);
        assert_approx_eq!(oracle_deficit_new, deficit, 0.001);

        // na chamada do wizard, no arranque do ciclo, o deficit do sector é atualizado com o que se calculou aí. (rebuild_run_secs)
        sec.deficit = deficit;

        // e no final da rega ou na interrupção/paragem, o deficit do setor é atualizado pelo valor regado - stop_sector
        sec.deficit -= minutes * sec.debit;
        d += 1;
    }
}

pub fn create_data(data_since: CtrlTime, db: &Persist, nr_of_days: u64, rain: f32) {
    // criar os dados base do dia em causa
    // temos que criar rain para os dias em causa
    let mut d = 0;
    while d < nr_of_days {
        for i in 0..24 {
            // registamos um valor de hora a hora, só para ver a coisa funcionar...porque o et é carregado á frente diretamente
            let time = data_since.add_days(d) + i * CtrlTime::NR_NANOS_IN_A_HOUR;
            let values = Values { today_rain: rain, temp: 20., wind_b: 0., wind_i: 10., hr: 0.5, press: 1000. };
            ins_sensor_data(time, values, db);
        }
        d += 1;
    }
    // temos que criar et para os dias em causa
    d = 0;
    let mut daily_metrics_buf = ArrayVec::<SensorValue, 730>::new();
    while d < nr_of_days {
        let time = data_since.add_days(d);
        daily_metrics_buf.push(SensorValue::new(Metric::EvapoTranspiration as u8, time, ORACLE_ET));
        d += 1;
    }
    _ = db.ins_daily_data_batch_aux(&daily_metrics_buf);
}

// cenário de arranque após algum tempo parado
#[test]
fn wtr_wizard_strategy_base_003() {
    let start_time = CtrlTime::from_utc_parts(1970, 01, 01, 1, 0, 0);

    let db = Persist::new();

    let mut wtr_cfg = WtrCfg::new(db.clone(), start_time);
    // live since ever - maior do que zero por causa de se utiizar o zero para validar que nunca correu
    wtr_cfg.live_since = CtrlTime::from_utc_parts(1970, 01, 01, 0, 0, 1);

    // criação das condições
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();

    let nr_of_days = 2; // nr de dias que vamos testar

    // criamos os dados
    create_data(start_time.sod_ux_e(), &db, nr_of_days, 0.);

    // ir buscar setores e estabelecer condições
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    let sec = &mut list[0];
    // o setor correu no dia anterior, ou seja, já correu
    sec.deficit = 0.;
    sec.last_watered_in = start_time;

    simulate_machine_run(nr_of_days, start_time, sec, &db, &wtr_cfg, true, 1.);

    // agora simulamos a paragem da máquina
    // a maquina arrancou a dia 1/Jan
    // correu 2 dias portanto terá parado 3/Jan
    // rearranca 10 dias depois portanto a 13/Jan
    let new_start_time = CtrlTime::from_utc_parts(1970, 01, 13, 1, 0, 0);
    // e vamos assumir que vai correr mais 2 dias (o nr_of_days lá de cima)
    wtr_cfg.live_since = new_start_time;

    // e testamos
    simulate_machine_run(nr_of_days, new_start_time, sec, &db, &wtr_cfg, false, 1.);
}

// cenário "normal" durante um ano (com et )
#[test]
fn wtr_wizard_strategy_base_004() {
    let start_time = CtrlTime::from_utc_parts(2022, 07, 1, 04, 30, 0);

    let db = Persist::new();

    let live_since = CtrlTime::from_utc_parts(2022, 01, 01, 10, 30, 1);
    let mut wtr_cfg = WtrCfg::new(db.clone(), live_since);
    // live since ever - maior do que zero por causa de se utiizar o zero para validar que nunca correu
    wtr_cfg.live_since = live_since;

    // criação das condições
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();

    let nr_of_days = 365; // nr de dias que vamos testar

    // criamos os dados
    create_data(live_since.sod_ux_e(), &db, nr_of_days * 2, 0.);

    // ir buscar setores e estabelecer condições
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    let sec = &mut list[0];

    // o setor correu no dia anterior, ou seja, já correu
    sec.deficit = 0.;
    sec.last_watered_in = start_time.sub_days(1);

    simulate_machine_run(nr_of_days, start_time, sec, &db, &wtr_cfg, true, 1.);
}

// cenário "normal" durante um ano (com et e com rain)
#[test]
fn wtr_wizard_strategy_base_005() {
    let start_time = CtrlTime::from_utc_parts(2022, 07, 1, 04, 30, 0);

    let db = Persist::new();

    let live_since = CtrlTime::from_utc_parts(2022, 01, 01, 10, 30, 1);
    let mut wtr_cfg = WtrCfg::new(db.clone(), live_since);
    // live since ever - maior do que zero por causa de se utiizar o zero para validar que nunca correu
    wtr_cfg.live_since = live_since;

    // criação das condições
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();

    let nr_of_days = 365; // nr de dias que vamos testar

    // criamos os dados
    create_data(live_since.sod_ux_e(), &db, nr_of_days * 2, 0.041666667); //em 24 horas dará 1 mm...esperemos :-)

    // ir buscar setores e estabelecer condições
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    let sec = &mut list[0];

    // o setor correu no dia anterior, ou seja, já correu
    sec.deficit = 0.;
    sec.last_watered_in = start_time.sub_days(1);

    simulate_machine_run(nr_of_days, start_time, sec, &db, &wtr_cfg, true, 1.);
}

// cenário "normal" durante uma semana (com et e com rain que evite a rega)
#[test]
fn wtr_wizard_strategy_base_006() {
    let start_time = CtrlTime::from_utc_parts(2022, 07, 1, 04, 30, 0);

    let db = Persist::new();

    let live_since = CtrlTime::from_utc_parts(2022, 01, 01, 10, 30, 1);
    let mut wtr_cfg = WtrCfg::new(db.clone(), live_since);
    // live since ever - maior do que zero por causa de se utiizar o zero para validar que nunca correu
    wtr_cfg.live_since = live_since;

    // criação das condições
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();

    let nr_of_days = 7; // nr de dias que vamos testar

    // criamos os dados
    create_data(start_time.sub_days(1).sod_ux_e(), &db, nr_of_days * 2, 0.41666667); //em 24 horas dará 10 mm...esperemos :-)

    // ir buscar setores e estabelecer condições
    let mut list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut list).unwrap();

    let sec = &mut list[0];

    // o setor correu no dia anterior, ou seja, já correu
    sec.deficit = 0.;
    sec.last_watered_in = start_time.sub_days(1);

    simulate_machine_run(nr_of_days, start_time, sec, &db, &wtr_cfg, true, 10.);
}

#[test]
fn wtr_new_day_program_start() {}

#[test]
fn wtr_new_day_day_0_to_6() {}

#[test]
fn wtr_new_day_week_change() {}

#[test]
fn wtr_new_day_with_rain() {}

#[test]
fn wtr_new_day_without_rain() {}

#[test]
fn wtr_new_day_with_et() {}

#[test]
fn wtr_new_day_without_et() {}

#[test]
fn wtr_new_day_with_rain_and_et() {}

#[test]
fn wtr_new_day_with_rain_no_et() {}

#[test]
fn wtr_new_day_with_et_no_rain() {}
