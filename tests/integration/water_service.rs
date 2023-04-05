#![allow(dead_code)]

use ctrl_lib::services::electronics::devices_svc::DevicesSvc;
use itertools::Combinations;
use itertools::Itertools;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::thread::*;
use std::time::Duration;
use std::time::Instant;

use crate::integration::db_irrigation::*;
use ctrl_lib::app_context::start_up::*;
use ctrl_lib::app_time::{ctrl_time::*, date_time::*, schedule::*, schedule_params::*, tm::*};
use ctrl_lib::config::{wthr_cfg::*, wtr_cfg::*, Module};
use ctrl_lib::data_structs::msgs::{alert::*, weather::*};
use ctrl_lib::data_structs::rega::{command::*, mode::*, running_ptr::*, state::*, watering_status::*};
use ctrl_lib::data_structs::sensor::{daily_value::*, stat_metric::*};
use ctrl_lib::db::{db_error::*, db_sql_lite::*};
use ctrl_lib::services::electronics::valve_state::*;
use ctrl_lib::services::irrigation::{cycle::*, cycle_type::*, db_model::*, sector::*, wtr_engine::*, wtr_svc::*};
use ctrl_lib::services::msg_broker::msg_brkr_svc::*;
use ctrl_lib::ArrayVec;
use ctrl_lib::{log_info, logger::*, utils::*};
use ctrl_prelude::{domain_types::*, globals::*};

pub trait DBModelMachineTest<'a>: DBModelRega<'a> {
    fn delete_daily_measures(&self) -> SimpleResult {
        let sql1: &str = "delete from sensor_data;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn delete_metrics(&self) -> SimpleResult {
        let sql1: &str = "delete from sensor_daily_data;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn delete_all(&self) -> SimpleResult {
        let sql1: &str = "delete from watered_sector;";
        let sql2: &str = "delete from watered_cycle;";
        let sql3: &str = "delete from scheduled_cycle;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();
        let _res = self.exec_prep(&mut stmt);

        let mut stmt = conn.prepare_cached(sql2).unwrap();
        _ = self.exec_prep(&mut stmt);

        let mut stmt = conn.prepare_cached(sql3).unwrap();
        self.exec_prep(&mut stmt)
    }

    fn update_mode_params(&self, mode: Mode) -> SimpleResult {
        let sql1: &str = "update mods_data set string=? where module=2 and param=3;";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();
        _ = stmt.raw_bind_parameter(1, mode.to_string());
        self.exec_prep(&mut stmt)
    }

    fn enable_all_sectors(&self) -> SimpleResult {
        let sql1: &str = "update sector set enabled = 1;";
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();
        self.exec_prep(&mut stmt)
    }
    ///Tuple corresponde as colunas por tipo de dados na tabela de parametros = (float, int, string)
    fn update_param_gen(&self, module: u8, param: u8, tupple: (Option<f64>, Option<u64>, Option<String>)) -> SimpleResult {
        let sqlfloat: &str = "update mods_data set float=? where module=? and param=?;";
        let sqlstring: &str = "update mods_data set string=? where module=? and param=?;";
        let sqlint: &str = "update mods_data set int=? where module=? and param=?;";

        let conn = &self.get_conn().conn;
        let mut stmtfloat = conn.prepare_cached(sqlfloat).unwrap();
        let mut stmtstring = conn.prepare_cached(sqlstring).unwrap();
        let mut stmtint = conn.prepare_cached(sqlint).unwrap();

        match tupple {
            (Some(float), _, _) => {
                let mut _res = stmtfloat.raw_bind_parameter(1, float);
                _res = stmtfloat.raw_bind_parameter(2, module);
                _res = stmtfloat.raw_bind_parameter(3, param);
                self.exec_prep(&mut stmtfloat)
            }
            (_, Some(int), _) => {
                let mut _res = stmtint.raw_bind_parameter(1, int);
                _res = stmtint.raw_bind_parameter(2, module);
                _res = stmtint.raw_bind_parameter(3, param);
                self.exec_prep(&mut stmtint)
            }
            (_, _, Some(string)) => {
                let mut _res = stmtstring.raw_bind_parameter(1, string);
                _res = stmtstring.raw_bind_parameter(2, module);
                _res = stmtstring.raw_bind_parameter(3, param);
                self.exec_prep(&mut stmtstring)
            }
            _ => Ok(()),
        }
    }

    fn get_mode_params(&self) -> Mode {
        let sql1: &str = "select string as mode from mods_data where module=2 and param=3;";

        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(sql1).unwrap();

        let mut mode_result: Option<String> = None;
        let mut rows = stmt.raw_query();
        while let Ok(Some(row)) = rows.next() {
            //REVIEW, isto está a passar os nulls...pelo que funciona se a bd não tiver nulos....
            mode_result = match row.get::<usize, String>(0) {
                Ok(val) => Some(val),
                Err(_) => None,
            }
        }
        Mode::from_str(&mode_result.unwrap()).unwrap()
    }
}

impl<'a> DBModelMachineTest<'a> for Persist {}

//>>>>>>>>>>>>>>>>>>>  TESTES COMEÇAM AQUI >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

#[test]
//máquina deve ir para o modo NO_SCHEDULE_DEF
fn wtr_start_no_sched_mode_standard_after_7_days_000_001() {
    log_info!("test start_no_sched_mode_standard_after_7_days started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);
    simple_no_run(time_ref, Mode::Standard, 0, State::NoScheduleDef, time_ref.sub_days(1));

    log_info!("test start_no_sched_mode_standard_after_7_days finished");
}

fn simple_no_run(time_ref: CtrlTime, mode: Mode, nr_of_cycles: usize, valid_state: State, last_save: CtrlTime) {
    let (db, start_up) = prepare_common(time_ref, mode, last_save);

    for i in 0..nr_of_cycles {
        insert_schedule(time_ref.add_days(i as u64), &db, format!("name {}", i), CycleType::Standard);
    }

    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_ref);

    assert_eq!(
        wtr_svc.engine.cycles.len(),
        nr_of_cycles + MAX_INTERNALS,
        "{} ciclo(s) criado(s) pelo teste + 2 ciclos internos criados automaticamente",
        nr_of_cycles
    );

    assert_start_params(db, mode, &wtr_svc, start_up, valid_state);

    unsafe {SHUTTING_DOWN = true};
    wtr_svc.terminate(time_ref.add_secs(1));
    msg_broker.terminate();
    let _res = handle_evt_mng.join();
}

fn assert_start_params(db: Persist, p_mode: Mode, wtr_svc: &WtrSvc, start_up: StartupData, p_valid_state: State) {
    let mode = db.get_mode_params();
    assert!(mode == p_mode, "mode");
    assert!(wtr_svc.engine.wtr_cfg.in_error == 0, "in error");
    assert_eq!(wtr_svc.engine.wtr_cfg.in_alert, 0, "in alert");
    assert!(wtr_svc.engine.wtr_cfg.state == p_valid_state, "state");
    assert_eq!(wtr_svc.engine.wtr_cfg.wizard_info.last_stress_control_time.ux_ts(), 0, "last_stress_control_time");
    assert!(wtr_svc.engine.wtr_cfg.last_saved >= start_up.start_date, "data last saved");
}

#[test]
//máquina deve ir para o modo NO_SCHEDULE_DEF
fn wtr_start_no_sched_mode_standard_before_7_days_000_002() {
    log_info!("test start_no_sched_mode_standard_before_7_days started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);
    simple_no_run(time_ref, Mode::Standard, 0, State::NoScheduleDef, time_ref.sub_days(1));

    log_info!("test start_no_sched_mode_standard_before_7_days finished");
}

#[test]
//máquina no modo manual, mesmo sem schedule definido vai para o Manual Wait
fn wtr_start_no_sched_mode_manual_000_003() {
    log_info!("test start_no_sched_mode_manual started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);

    simple_no_run(time_ref, Mode::Manual, 0, State::ManWait, time_ref.sub_days(1));

    log_info!("test start_no_sched_mode_manual finished");
}

#[test]
//máquina deve ir para o modo WAIT, depois de criar o ou os ciclos que forem necessários
fn wtr_start_no_sched_mode_wizard_000_004() {
    log_info!("test start_no_sched_mode_wizard started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);
    simple_no_run(time_ref, Mode::Wizard, 1, State::WzrWait, time_ref.sub_days(1));

    log_info!("test start_no_sched_mode_wizard finished");
}

fn terminate_and_wait(mut wtr_svc: WtrSvc, time: CtrlTime, msg_broker: Arc<MsgBrkr>, handle_evt_mng: std::thread::JoinHandle<()>) {
    wtr_svc.terminate(time);
    assert!(wtr_svc.engine.wtr_cfg.state == State::Shutdown, "state");
    unsafe {
        SHUTTING_DOWN = true;
    }
    msg_broker.terminate();
    let _res = handle_evt_mng.join();
}

#[test]
//máquina deve ir para o modo MANUAL WAIT
fn wtr_start_with_sched_mode_manual_000_005() {
    log_info!("test start_with_sched_mode_manual started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);
    simple_no_run(time_ref, Mode::Manual, 1, State::ManWait, time_ref.sub_days(1));

    log_info!("test start_with_sched_mode_manual finished");
}

pub fn insert_schedule(time_ref: CtrlTime, db: &Persist, name: String, cycle_type: CycleType) {
    let schedule = Schedule::build_run_forever(time_ref, 2, ScheduleRepeatUnit::Days);
    let mut cycle = Cycle { schedule, cycle_type, ..Default::default() };
    cycle.name = name;
    let _res = db.ins_cycle(&mut cycle);
}

#[test]
//máquina deve ir para o modo STANDARD WAIT
fn wtr_start_with_sched_mode_standard_000_006() {
    log_info!("test start_with_sched_mode_standard started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 5, 19, 0, 0);

    simple_no_run(time_ref, Mode::Standard, 1, State::StdWait, time_ref.sub_days(1));

    log_info!("test start_with_sched_mode_standard finished");
}

#[test]
// máquina deve ir para o modo WAIT WIZARD
// a máquina se não tiver schedule wizard vai criar, mas se já existir criado, não cria um novo
fn wtr_start_with_sched_mode_wizard_with_prev_sched_000_007() {
    log_info!("test start_with_sched_mode_wizard started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, time_ref.sub_days(1));

    insert_schedule(time_ref, &db, String::from("Wizard-auto"), CycleType::Wizard);

    let (msg_broker, handle_evt_mng, wtr_svc) = create_objects(&db, time_ref);

    let cycles = wtr_svc.cycles_clone();
    assert_eq!(cycles.len(), MAX_INTERNALS, "Apenas os ciclos internos");
    // a regra é o last save e não o last run :-) pelo que aqui não inicializa o week acc water nem o week acc counter
    assert_start_params(db, Mode::Wizard, &wtr_svc, start_up, State::WzrWait);

    terminate_and_wait(wtr_svc, time_ref.add_secs(1), msg_broker, handle_evt_mng);

    log_info!("test start_with_sched_mode_wizard finished");
}

pub fn create_objects(db: &Persist, time_ref: CtrlTime) -> (Arc<MsgBrkr>, JoinHandle<()>, WtrSvc) {
    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    let weather_cfg = arc_rw(WthrCfg::new(db.clone(), time_ref));
    let dev_svc = Arc::new(DevicesSvc::new(db, msg_broker.clone()));
    let wtr_svc = WtrSvc::new(weather_cfg.read().alrt_thresholds.clone(), msg_broker.clone(), db.clone(), time_ref, dev_svc);
    (msg_broker, handle_evt_mng, wtr_svc)
}

#[test]
// deve criar o ciclo automaticamente e
// deve ir para o modo WAIT WIZARD
fn wtr_start_with_sched_mode_wizard_no_prev_sched_000_008() {
    log_info!("test start_with_sched_mode_wizard started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, time_ref.sub_days(1));

    let (msg_broker, handle_evt_mng, wtr_svc) = create_objects(&db, time_ref);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "apenas os ciclos internos");

    // aqui a maquina não recebeu o comando de start, pelo que o week acc counter não foi inicializado para o dia da semana,
    // nem o week acc water a 0, apesar de a ultima rega ter sido á mais de 7 dias
    assert_start_params(db, Mode::Wizard, &wtr_svc, start_up, State::WzrWait);

    terminate_and_wait(wtr_svc, time_ref.add_secs(1), msg_broker, handle_evt_mng);

    log_info!("test start_with_sched_mode_wizard finished");
}

// -------- MODE STANDARD
// ver o arranque com um ciclo cujo tempo já passou.
// como o ciclo é never, não mexe na data do ciclo, e ficará em standard wait para sempre mas nunca executa nada
//
#[test]
fn wtr_start_standard_one_schedule_never_in_the_past_000_009() {
    log_info!("test start_standard_one_schedule_never_in_the_past started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let (db, start_up) = prepare_common(time_ref, Mode::Standard, time_ref.sub_days(2));

    let mut schedule = Schedule::build_run_forever(time_ref.sub_days(2), 2, ScheduleRepeatUnit::Days);
    schedule.repeat_kind = ScheduleRepeat::Never;
    let mut cycle = Cycle { schedule, cycle_type: CycleType::Standard, ..Default::default() };
    cycle.name = String::from("standard 1");
    let _res = db.ins_cycle(&mut cycle);

    let (msg_broker, handle_evt_mng, wtr_svc) = create_objects(&db, time_ref);

    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "ciclos no total");
    assert_eq!(wtr_svc.engine.std_ptrs.len(), 1, "1 ciclo standard");

    let new_start = &wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert!(&time_ref.sub_days(2) == new_start, "datas de arranque");
    // last save  á menos de 7 dias pelo que não inicializa week acc water nem week acc conter
    assert_start_params(db, Mode::Standard, &wtr_svc, start_up, State::StdWait);

    terminate_and_wait(wtr_svc, time_ref.add_secs(1), msg_broker, handle_evt_mng);

    log_info!("test start_standard_one_schedule_never_in_the_past finished");
}

fn assert_cycle_start_and_first_sector_running_enhanced(time_tick: CtrlTime, wtr_svc: &WtrSvc, start_delta: u64) {
    let mut next: u64 = 0;
    let secs_to_wtr = wtr_svc.engine.run_secs.len();
    for run_sec_id in 0..secs_to_wtr {
        let rs = &wtr_svc.engine.run_secs[run_sec_id];
        let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];
        if wtr_svc.engine.wtr_cfg.mode != Mode::Wizard {
            assert!(
                (rs.wtr_tgt_min - sec.max_duration).abs() < f32::EPSILON,
                "target de minutos para a rega - setor {} - left: {}, right: {}",
                &sec.name,
                rs.wtr_tgt_min,
                sec.max_duration
            );

            if run_sec_id < 1 {
                assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", sec.id);
            } else {
                assert_eq!(rs.status, WateringStatus::Waiting, "estado da rega do setor {}", sec.id);
            }
            assert!(
                (time_tick + next).0 as f64 - rs.start.add_secs(start_delta).0 as f64 <= GIGA_F,
                "sector start: left: {}  right: {}",
                rs.start.add_secs(start_delta).as_rfc3339_str_e(),
                (time_tick + next).as_rfc3339_str_e()
            );
            assert!(
                (rs.end.0 as f64 - (rs.start + min_to_nano(rs.wtr_tgt_min)).0 as f64).abs() < GIGA_F,
                "setor end.  left: {} right: {}",
                rs.end.as_rfc3339_str_e(),
                (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e()
            );
            next = next + (sec.max_duration * 60. * GIGA_F as f32) as u64 + wtr_svc.engine.wtr_cfg.pump_recycle_time as u64 * GIGA_U;
        } else {
            if run_sec_id < 1 {
                assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", sec.id);
            } else {
                assert_eq!(rs.status, WateringStatus::Waiting, "estado da rega do setor {}", sec.id);
            }
            assert!(
                (rs.end.0 as f64 - (rs.start + min_to_nano(rs.wtr_tgt_min)).0 as f64).abs() < GIGA_F,
                "setor end.  left: {} right: {}",
                rs.end.as_rfc3339_str_e(),
                (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e()
            );
            next = next + (rs.wtr_tgt_min * 60. * GIGA_F as f32) as u64 + wtr_svc.engine.wtr_cfg.pump_recycle_time as u64 * GIGA_U;
        }
    }
}

fn establish_sectors_base_condition_before_cycle_start(wtr_svc: &mut WtrSvc) {
    for i in 0..MAX_SECTORS {
        let sec = &mut wtr_svc.engine.sectors[i];
        sec.deficit = 0.;
        sec.last_watered_in = CtrlTime(0);
        sec.last_change = CtrlTime(0);
    }
}

// ver o arranque com um ciclo cujas condições são para correr
// e todos os ciclos são para executar mas acontece um erro a meio entre o ciclo 2 e o ciclo 3
// a máquina deve ir para o estado erro e não reagir a mais comandos até se colocar de novo em manual e depois no modo pretendido
#[test]
fn wtr_start_standard_one_schedule_run_all_sectors_with_error_000_010() {
    log_info!("test start_standard_one_schedule_run_all_sectors_with_error started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    for sec in 0..MAX_SECTORS {
        run_all_sector_with_error_on_sector_n(time_ref, prev_water, sec);
    }
    log_info!("test start_standard_one_schedule_run_all_sectors_with_error finished");
}

fn run_all_sector_with_error_on_sector_n(time_ref: CtrlTime, prev_water: CtrlTime, sec_in_err: usize) {
    let (db, start_up, prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);
    let cycles = wtr_svc.cycles_clone();
    assert_eq!(cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    // como tenho vários testes a correr em sequencia, que não limpam os dados no fim, temos que assegurar que os dados estão coerentes para não dar erro nas contas das datas
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }
    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();
    let new_start = &wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&prev_water.add_days(22).as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");
    // como a ultima vez que regou foi á mais de 7 dias (ver regra no código) , inicializou o water acc a 100, mas é reinicializado para 0
    assert_start_params(db, Mode::Standard, &wtr_svc, start_up, State::StdWait);
    let mut time_tick = *new_start;
    //em tese isto arranca a rega
    process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> processo de arranque + cycle start -> one schedule -> all sectors"));
    assert_eq!(wtr_svc.engine.active_ptrs.cycle.unwrap(), cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo 0 ativo - fora do loop");
    // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
    // mas apanhei aqui um tema do terminate a meio de qq coisa que tem que ser testado
    for sec_id in 0..MAX_SECTORS {
        let idx = sec_id as usize;
        // sabemos que estão 6 setores na BD
        // no primeiro já ficou ativado .com o time_tick de fora do ciclo, ou com o ultimo time_tick do loop
        if sec_id < sec_in_err {
            assert_active_sec_open(&wtr_svc, sec_id);

            time_tick = advance_to_end_of_sector(&wtr_svc, idx, time_tick);

            process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> all sectors"));
            assert_sec_closed(&wtr_svc, sec_id);
        }
        if sec_id == sec_in_err {
            //vamos simular um erro
            wtr_svc.snd_command(Command::Error);
        }
        if sec_id < MAX_SECTORS - 1 {
            time_tick = wtr_svc.engine.run_secs[idx + 1].start;
            process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> all sectors"));
        }
    }
    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
}

fn assert_sec_closed(wtr_svc: &WtrSvc, i: usize) {
    let valve_state = wtr_svc.engine.sectors[i].state.clone();
    assert!(valve_state == RelayState::Closed, "setor {} parou", i);
    assert!(wtr_svc.engine.active_ptrs.sec_id.is_none(), "limpou o active sector: {}", i);
    println!("setor {} parou", i);
}

fn assert_active_sec_open(wtr_svc: &WtrSvc, expected_sec_id: usize) -> usize {
    let sec_id = wtr_svc.engine.active_ptrs.sec_id.as_ref().unwrap();
    assert_eq!(*sec_id, expected_sec_id as u8, "sector {} ativo", expected_sec_id);
    let valve_state = wtr_svc.engine.sectors[*sec_id as usize].state.clone();
    assert!(valve_state == RelayState::Open, "setor {} a regar", *sec_id);
    assert_eq!(wtr_svc.engine.active_ptrs.sec_id.unwrap(), expected_sec_id as u8, "o active sector é o: {}", expected_sec_id);
    println!("setor {} a regar", sec_id);
    *sec_id as usize
}

fn process_tick(wtr_svc: &mut WtrSvc, time_tick: CtrlTime, desc: &String) {
    let t0 = Instant::now();
    wtr_svc.verify_things_to_do(time_tick);
    println!("water machine -> {}: {}", desc, elapsed_dyn(t0.elapsed().as_nanos() as u64));
}

fn process_tick_in_proc(wtr_svc: &mut WtrSvc, time_tick: CtrlTime, desc: &String, nr_skipped: usize) {
    let t0 = Instant::now();
    wtr_svc.verify_things_to_do(time_tick);
    println!("water machine -> {} -> skip {} sectors: {}", desc, nr_skipped, elapsed_dyn(t0.elapsed().as_nanos() as u64));
}
// ver o arranque com um ciclo cujas condições são para correr
// e todos os ciclos são para executar mas acontece um erro a meio e depois recupera-se
// o comportamento esperado é que se puxa a máquina para o modo standard e permance em standard wait, esperando o proximo ciclo
// no procimo ciclo executa sem problemas.
#[test]
fn wtr_start_standard_one_schedule_run_all_sectors_with_error_with_recover_000_011() {
    log_info!("test start_standard_one_schedule_run_all_sectors_with_error_with_recover started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    for sec in 0..MAX_SECTORS {
        run_all_sectors_with_error_on_sector_n_and_recover(time_ref, prev_water, sec);
    }
    log_info!("test start_standard_one_schedule_run_all_sectors_with_error_with_recover finished");
}

fn run_all_sectors_with_error_on_sector_n_and_recover(time_ref: CtrlTime, prev_water: CtrlTime, sec_in_err: usize) {
    let (db, start_up, prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    // como tenho vários testes a correr em sequencia, que não limpam os dados no fim, temos que assegurar que os dados estão coerentes para não dar erro nas contas das datas
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();
    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&prev_water.add_days(22).as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");
    // water week acc = 0 porque regou á mais de 7 dias
    assert_start_params(db, Mode::Standard, &wtr_svc, start_up, State::StdWait);
    let mut time_tick = new_start;
    process_tick(&mut wtr_svc, time_tick, &format!("mode {} -> start + cycle start -> one schedule -> all sectors", "standard"));
    let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
    assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo {} ativo - fora do loop", cycle_ptr);
    // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
    // mas apanhei aqui um tema do terminate a meio de qq coisa que tem que ser testado
    for idx in 0..MAX_SECTORS {
        // sabemos que estão 6 setores na BD
        // no primeiro já ficou ativado .com o time_tick de fora do ciclo, e os outros são ativados com o ultimo time_tick do loop
        if idx < sec_in_err {
            assert_active_sec_open(&wtr_svc, idx);

            time_tick = advance_to_end_of_sector(&wtr_svc, idx, time_tick);

            process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> all sectors"));

            assert_sec_closed(&wtr_svc, idx);
        }
        if idx == sec_in_err {
            //vamos simular um erro
            wtr_svc.snd_command(Command::Error);
        }

        if idx < MAX_SECTORS - 1 {
            time_tick = wtr_svc.engine.run_secs[idx + 1].start;
            process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> all sectors"));
        }
    }
    println!("A máquina está no tempo: {}", time_tick.as_rfc3339_str_e());
    // Agora tentamos a recuperação
    let t0 = Instant::now();
    wtr_svc.snd_command(Command::ChangeMode(Mode::Standard));
    println!("send command change mode: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    //avançamos 2 horas - 6 setores a 30 min. cada um dá 3 horas.
    //quer dizer que apanhamos a meio do ciclo, mas como o modo standard só valida o inicio do ciclo, vai fazer o reschedule do ciclo para o dia seguinte
    time_tick = time_tick.add_secs_f32(60. * 60. * 2.);
    println!("A máquina está no tempo: {}", time_tick.as_rfc3339_str_e());

    process_tick(&mut wtr_svc, time_tick, &String::from("process time tick"));
    let new_start = new_start.add_days(2);
    let cycle_start = &wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&cycle_start.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "nova data de arranque após recuperação");
    assert!(wtr_svc.get_mode() == Mode::Standard, "mode after recover");
    assert!(wtr_svc.engine.wtr_cfg.state == State::StdWait, "state after recover");
    time_tick = time_tick.add_secs(1800);
    //avançamos + 12 horas e não deve mudar o modo
    process_tick(&mut wtr_svc, time_tick, &String::from("advance time"));
    assert!(wtr_svc.get_mode() == Mode::Standard, "mode while waiting");
    assert!(wtr_svc.engine.wtr_cfg.state == State::StdWait, "state while waiting recover");
    //e agora vamos reexecutar o ciclo no periodo/dia seguinte
    time_tick = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, &Vec::<bool>::new(), cycle_ptr, State::StdWait, Mode::Standard, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
}

fn advance_to_end_of_sector(wtr_svc: &WtrSvc, run_sec_ptr: usize, time_tick: CtrlTime) -> CtrlTime {
    let tempo_rega = wtr_svc.engine.run_secs[run_sec_ptr].wtr_tgt_min * 60.;
    //saltamos para o fim
    time_tick.add_secs_f32(tempo_rega)
}

fn get_nth_standard_cycle(wtr_svc: &WtrSvc, nth: usize) -> Option<usize> {
    Some(wtr_svc.engine.std_ptrs[nth - 1].1 as usize)
}

fn prepare_standard(start_time: CtrlTime, prev_water: CtrlTime, mode: Mode) -> (Persist, StartupData, CtrlTime, SMsgBrkr, JoinHandle<()>, WtrSvc) {
    let (db, start_up) = prepare_common(start_time, mode, prev_water);

    insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);

    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();

    //criamos a maquina um pouco antes das 22 do dia seguinte ao arranque para calcular o tempo do ciclo para o dia certo
    let time_tick = start_time.add_secs(85000);
    let weather_cfg = arc_rw(WthrCfg::new(db.clone(), start_time));
    let dev_svc = Arc::new(DevicesSvc::new(&db, msg_broker.clone()));
    let wtr_svc = WtrSvc::new(weather_cfg.read().alrt_thresholds.clone(), msg_broker.clone(), db.clone(), time_tick, dev_svc);

    (db, start_up, prev_water, msg_broker, handle_evt_mng, wtr_svc)
}

fn prepare_standard_no_mangle(
    start_time: CtrlTime, prev_water: CtrlTime, mode: Mode,
) -> (Persist, StartupData, CtrlTime, SMsgBrkr, JoinHandle<()>, WtrSvc) {
    let (db, start_up) = prepare_common(start_time, mode, prev_water);

    insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);

    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    let dev_svc = Arc::new(DevicesSvc::new(&db, msg_broker.clone()));
    let weather_cfg = arc_rw(WthrCfg::new(db.clone(), start_time));
    let wtr_svc = WtrSvc::new(weather_cfg.read().alrt_thresholds.clone(), msg_broker.clone(), db.clone(), start_time, dev_svc);

    (db, start_up, prev_water, msg_broker, handle_evt_mng, wtr_svc)
}

/// devolve a (db, startup, week_acc_counter)
pub fn prepare_common(time_ref: CtrlTime, mode: Mode, last_save: CtrlTime) -> (Persist, StartupData) {
    let db = Persist::new();
    setup_start_time(time_ref);
    let start_up: StartupData = StartupData::build(time_ref);
    let _ = db.delete_all();
    let _ = db.update_mode_params(mode);
    // para simular que a ultima vez que executou foi á 4 dias antes da data do arranque
    let _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(last_save.ux_ts()), None));

    let _ = db.enable_all_sectors();
    (db, start_up)
}

fn get_disabled_positions(secs_to_skip: &Vec<bool>) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::with_capacity(MAX_SECTORS);
    for i in 0..secs_to_skip.len() {
        if !secs_to_skip[i] {
            result.push(i);
        }
    }
    result
}

fn process_sectors_enhanced(
    wtr_svc: &mut WtrSvc, time_tick: CtrlTime, secs_to_skip: &Vec<bool>, cycle_ptr: usize, expected_wait_state: State, mode: Mode, start_delta: u64,
) -> CtrlTime {
    let disabled = get_disabled_positions(secs_to_skip);
    let mut time_tick = time_tick;
    let nr_skipped = disabled.len();

    process_tick_in_proc(wtr_svc, time_tick, &format!("mode {mode} -> start + cycle start -> one schedule "), disabled.len());

    // se se saltarem todos os setores, não á nada a fazer
    if nr_skipped < MAX_SECTORS {
        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
        println!("ciclo {} ativo", cycle_ptr);

        assert_cycle_start_and_first_sector_running_enhanced(time_tick, &wtr_svc, start_delta);

        let nr_secs_to_wtr = wtr_svc.engine.run_secs.len();
        for run_sec_id in 0..nr_secs_to_wtr {
            let rs = &wtr_svc.engine.run_secs[run_sec_id];
            let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];

            let sec_id = assert_active_sec_open(&wtr_svc, sec.id as usize);
            time_tick = advance_to_end_of_sector(&wtr_svc, run_sec_id, time_tick);

            process_tick_in_proc(wtr_svc, time_tick, &format!("mode {} -> sector end -> one schedule ", mode), nr_skipped);

            assert_sec_closed(&wtr_svc, sec_id);

            if run_sec_id < nr_secs_to_wtr - 1 {
                time_tick = wtr_svc.engine.run_secs[run_sec_id + 1].start;
                process_tick_in_proc(wtr_svc, time_tick, &format!("mode {} -> sector start -> one schedule ", mode), nr_skipped);
            }
        }
    }
    assert!(wtr_svc.engine.wtr_cfg.state == expected_wait_state, "state");
    time_tick
}

// let it1_1 = (0..MAX_SECTORS).combinations(0);
// let it1_2 = (0..MAX_SECTORS).combinations(1);
// let it1_3 = (0..MAX_SECTORS).combinations(2);
// let it1_4 = (0..MAX_SECTORS).combinations(3);
// let it1_5 = (0..MAX_SECTORS).combinations(4);
// let it1_6 = (0..MAX_SECTORS).combinations(5);
// // let it2 = (0..MAX_SECTORS).permutations(MAX_SECTORS);

// println!("{:?}", it1_1.collect::<Vec<Vec<usize>>>());
// println!("{:?}", it1_2.collect::<Vec<Vec<usize>>>());
// println!("{:?}", it1_3.collect::<Vec<Vec<usize>>>());
// println!("{:?}", it1_4.collect::<Vec<Vec<usize>>>());
// println!("{:?}", it1_5.collect::<Vec<Vec<usize>>>());
// println!("{:?}", it1_6.collect::<Vec<Vec<usize>>>());

// corremos todas as combinações possiveis de setores enabled/disabled , que para os 6 setores são 63
// [[]]	                                                                                                                        => 1
// [[0], [1], [2], [3], [4], [5]]                                                                                               => 6
// [[0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [1, 2], [1, 3], [1, 4], [1, 5], [2, 3], [2, 4], [2, 5], [3, 4], [3, 5], [4, 5]]     => 15
// [[0, 1, 2], [0, 1, 3], [0, 1, 4], [0, 1, 5], [0, 2, 3], [0, 2, 4], [0, 2, 5], [0, 3, 4], [0, 3, 5], [0, 4, 5], [1, 2, 3],
//  [1, 2, 4], [1, 2, 5], [1, 3, 4], [1, 3, 5], [1, 4, 5], [2, 3, 4], [2, 3, 5], [2, 4, 5], [3, 4, 5]]                          => 20
// [[0, 1, 2, 3], [0, 1, 2, 4], [0, 1, 2, 5], [0, 1, 3, 4], [0, 1, 3, 5], [0, 1, 4, 5], [0, 2, 3, 4],
//  [0, 2, 3, 5], [0, 2, 4, 5], [0, 3, 4, 5], [1, 2, 3, 4], [1, 2, 3, 5], [1, 2, 4, 5], [1, 3, 4, 5], [2, 3, 4, 5]]             => 15
// [[0, 1, 2, 3, 4], [0, 1, 2, 3, 5], [0, 1, 2, 4, 5], [0, 1, 3, 4, 5], [0, 2, 3, 4, 5], [1, 2, 3, 4, 5]]                       => 6
#[test]
fn wtr_start_standard_one_schedule_run_skip_n_sectors_000_012_enahnced() {
    log_info!("test wtr_start_standard_one_schedule_run_skip_n_sectors_000_012_enahnced started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    let mut combinations: Combinations<Range<usize>>;

    for nr_secs in 0..MAX_SECTORS {
        combinations = (0..MAX_SECTORS).combinations(nr_secs);
        for v in combinations {
            let mut secs_to_skip = vec![true, true, true, true, true, true];
            for idx in v {
                secs_to_skip[idx] = false;
            }
            println!("{:?}", &secs_to_skip);
            run_skip_enhanced(time_ref, prev_water, &secs_to_skip);
            drop(secs_to_skip);
        }
    }
    log_info!("test wtr_start_standard_one_schedule_run_skip_n_sectors_000_018 finished");
}

fn run_skip_enhanced(time_ref: CtrlTime, prev_water: CtrlTime, secs_to_skip: &Vec<bool>) {
    let (db, start_up, prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);
    //vamos colocar os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &secs_to_skip, prev_water);

    // como tenho vários testes a correr em sequencia, que não limpam os dados no fim, temos que assegurar que os dados estão coerentes para não dar erro nas contas das datas
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");
    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&prev_water.add_days(22).as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);
    let mut time_tick = new_start;

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, secs_to_skip, cycle_ptr, State::StdWait, Mode::Standard, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
}

fn cfg_sectors_enabled(wtr_svc: &mut WtrSvc, flags: &[bool], prev_water: CtrlTime) {
    wtr_svc.engine.sectors[0].enabled = flags[0];
    //sector 0 disabled - o primeiro da lista passa assim a ser o 1
    wtr_svc.engine.sectors[1].enabled = flags[1];
    //sector 0 disabled - o primeiro da lista passa assim a ser o 2
    wtr_svc.engine.sectors[2].enabled = flags[2];
    //sector 0 disabled - o primeiro da lista passa assim a ser o 3
    wtr_svc.engine.sectors[3].enabled = flags[3];
    //sector 0 disabled - o primeiro da lista passa assim a ser o 4
    wtr_svc.engine.sectors[4].enabled = flags[4];
    //sector 0 disabled - o primeiro da lista passa assim a ser o 5
    wtr_svc.engine.sectors[5].enabled = flags[5];
    //sector 0 disabled - o primeiro da lista passa assim a ser nenhum - no cycle
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = prev_water;
    }
}

//
// o ciclo começa e a meio chega um alerta - não deve acontecer nada,
// isto porque so o modo wizard reage aos alertas
#[test]
fn wtr_start_standard_run_suspend_with_wind_alert_000_013() {
    for sec in 0..MAX_SECTORS {
        standard_try_suspend_wind_alert(AlertType::WIND, sec);
    }
}

fn standard_try_suspend_wind_alert(alert_type: AlertType, alert_on_sec: usize) {
    log_info!("test standard_try_suspend_wind_alert started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&prev_water.add_days(22).as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);
    establish_sectors_base_condition_before_cycle_start(&mut wtr_svc);

    //validar o estado inicial dos setores
    let mut time_tick = new_start;
    process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> start + cycle start -> one schedule -> try suspend alert"));
    // e aqui é que os tempos dos setores devem estar certos
    validate_sec_times(new_start, &wtr_svc);

    let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
    assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo 0 ativo - fora do loop");
    // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
    // mas apanhei aqui um tema do terminate a meio de qq coisa que tem que ser testado
    for i in 0..MAX_SECTORS {
        let idx = i as usize;
        // sabemos que estão 6 setores na BD
        // no primeiro já ficou ativado .com o time_tick de fora do ciclo, ou com o ultimo time_tick do loop
        assert_active_sec_open(&wtr_svc, i);

        time_tick = advance_to_end_of_sector(&wtr_svc, idx, time_tick);

        process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> try suspend alert"));
        assert_sec_closed(&wtr_svc, i);

        if i < MAX_SECTORS - 1 {
            time_tick = wtr_svc.engine.run_secs[idx + 1].start;
            process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> try suspend alert"));
        }
        if i == alert_on_sec {
            //enviar alerta
            wtr_svc.process_alert(Alert::new(alert_type.clone(), 25.)); //prioridade á máquina da rega
        }
    }
    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test standard_try_suspend_wind_alert finished");
}

// o ciclo começa e a meio chega um alerta - não deve acontecer nada, para além de informar o cliente das condições meteo
// isto porque so o modo wizard reage aos alertas
#[test]
fn wtr_start_standard_run_suspend_with_rain_alert_000_014() {
    for sec in 0..MAX_SECTORS {
        standard_try_suspend_wind_alert(AlertType::RAIN, sec);
    }
}

// isto só aplica no modo standard
// no modo wizard vai sempre para o wait porque cria o ciclo automaticamente caso não exista
// no modo manual vai sempre para o modo wait, para permitir criar os ciclos
#[test]
fn wtr_standard_no_sched_def_try_cmd_with_no_effect_000_015() {
    log_info!("test no_sched_def_try_cmd_with_no_effect started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Standard, prev_water);

    let time_tick = time_ref.add_secs(1);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclos");

    let mut time_tick = time_ref.add_secs(1);

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::NoScheduleDef);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::ChangeState;
    // cmd = Command::Error;
    let cmds = build_cmds([0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Standard);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::NoScheduleDef, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test no_sched_def_try_cmd_with_no_effect finished");
}

#[test]
fn wtr_standard_error_try_cmd_with_no_effect_000_016() {
    log_info!("test standard_error_try_cmd_with_no_effect started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Standard, prev_water);

    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclos standard");

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::NoScheduleDef);

    let mut time_tick = time_ref.add_secs(1);

    //colocamos a máquina em erro
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::Error;
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Standard);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::Error, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test standard_error_try_cmd_with_no_effect finished");
}

/// adiciona 1 segundo ao tempo
fn send_and_apply_command(wtr_svc: &mut WtrSvc, time_tick: CtrlTime, cmd: Command, valid_state: State) -> CtrlTime {
    wtr_svc.snd_command(cmd);
    let time_tick = time_tick.add_secs(1);
    wtr_svc.verify_things_to_do(time_tick);
    assert!(wtr_svc.engine.wtr_cfg.state == valid_state);
    time_tick
}

#[test]
fn wtr_wizard_error_try_cmd_with_no_effect_000_017() {
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, prev_water);

    //criamos a maquina um pouco antes das 22 do dia 2 para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclos standard");

    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    let mut time_tick = time_ref.add_secs(1);

    //colocamos a máquina em erro
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::Error;
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Wizard);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::Error, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect finished");
}

#[test]
fn wtr_manual_error_try_cmd_with_no_effect_000_018() {
    log_info!("test wtr_manual_error_try_cmd_with_no_effect started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Manual, State::ManWait);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::Error;
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Manual);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::Error, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_try_cmd_with_no_effect finished");
}

#[test]
fn wtr_manual_error_try_cmd_with_no_effect_with_std_cycles_000_019() {
    log_info!("test wtr_manual_error_try_cmd_with_no_effect started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Manual, State::ManWait);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::Error;
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Manual);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::Error, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_try_cmd_with_no_effect finished");
}

fn explore_error_state(p_mode: Mode, p_state: State) -> (Arc<MsgBrkr>, JoinHandle<()>, WtrSvc, CtrlTime) {
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up) = prepare_common(time_ref, p_mode, prev_water);

    //criamos a maquina um pouco antes das 22 do dia 2 para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclos standard");
    assert_start_params(db.clone(), p_mode, &wtr_svc, start_up.clone(), p_state);
    let mut time_tick = time_ref.add_secs(1);

    //colocamos a máquina em erro
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);

    (msg_broker, handle_evt_mng, wtr_svc, time_tick)
}

fn explore_error_state_with_std_cycles(p_mode: Mode, p_state: State) -> (Arc<MsgBrkr>, JoinHandle<()>, WtrSvc, CtrlTime) {
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up) = prepare_common(time_ref, p_mode, prev_water);

    insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);

    //criamos a maquina um pouco antes das 22 do dia 2 para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "com 1 ciclo standard");
    assert_start_params(db.clone(), p_mode, &wtr_svc, start_up.clone(), p_state);
    let mut time_tick = time_ref.add_secs(1);

    //colocamos a máquina em erro
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);

    (msg_broker, handle_evt_mng, wtr_svc, time_tick)
}

// aqui em tese só o mode manual funciona...ou não?  Nada impede de ir para qualquer outro modo...digo eu .. testar
#[test]
fn wtr_standard_error_apply_change_mode_manual_000_020() {
    log_info!("test wtr_standard_error_apply_change_mode_manual started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Standard, State::NoScheduleDef);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_manual finished");
}

#[test]
fn wtr_standard_error_apply_change_mode_manual_with_std_cycles_000_021() {
    log_info!("test wtr_standard_error_apply_change_mode_manual_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Standard, State::StdWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_manual_with_std_cycles finished");
}
// aqui deve ir para o NoSchedDef porque o explore error state não não têm ciclos standard definidos
#[test]
fn wtr_standard_error_apply_change_mode_standard_000_022() {
    log_info!("test wtr_standard_error_apply_change_mode_standard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Standard, State::NoScheduleDef);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::NoScheduleDef);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_standard finished");
}

#[test]
fn wtr_standard_error_apply_change_mode_standard_with_std_cycles_000_023() {
    log_info!("test wtr_standard_error_apply_change_mode_standard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Standard, State::StdWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::StdWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_standard finished");
}

#[test]
fn wtr_standard_error_apply_change_mode_wizard_000_024() {
    log_info!("test wtr_standard_error_apply_change_mode_wizard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Standard, State::NoScheduleDef);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_wizard finished");
}

#[test]
fn wtr_standard_error_apply_change_mode_wizard_with_std_cycles_000_025() {
    log_info!("test wtr_standard_error_apply_change_mode_wizard_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Standard, State::StdWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_error_apply_change_mode_wizard_with_std_cycles finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_manual_000_026() {
    log_info!("test wtr_manual_error_apply_change_mode_manual started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_manual finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_manual_with_std_cycles_000_027() {
    log_info!("test wtr_manual_error_apply_change_mode_manual_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_manual_with_std_cycles finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_standard_with_no_sched_000_028() {
    log_info!("test wtr_manual_error_apply_change_mode_standard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::NoScheduleDef);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_standard finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_standard_with_std_cycles_000_029() {
    log_info!("test wtr_manual_error_apply_change_mode_standard_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::StdWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_standard_with_std_cycles finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_wizard_000_030() {
    log_info!("test wtr_manual_error_apply_change_mode_wizard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_wizard finished");
}

#[test]
fn wtr_manual_error_apply_change_mode_wizard_with_std_cycles_000_031() {
    log_info!("test wtr_manual_error_apply_change_mode_wizard_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Manual, State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_error_apply_change_mode_wizard_with_std_cycles finished");
}
#[test]
fn wtr_wizard_error_apply_change_mode_manual_000_032() {
    log_info!("test wtr_wizard_error_apply_change_mode_manual started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_manual finished");
}

#[test]
fn wtr_wizard_error_apply_change_mode_manual_with_std_cycles_000_033() {
    log_info!("test wtr_wizard_error_apply_change_mode_manual_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_manual_with_std_cycles finished");
}

#[test]
fn wtr_wizard_error_apply_change_mode_standard_000_034() {
    log_info!("test wtr_wizard_error_apply_change_mode_standard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::NoScheduleDef);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_standard finished");
}

#[test]
fn wtr_wizard_error_apply_change_mode_standard_with_std_cycles_000_035() {
    log_info!("test wtr_wizard_error_apply_change_mode_standard_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::StdWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_standard_with_std_cycles finished");
}
#[test]
fn wtr_wizard_error_apply_change_mode_wizard_000_036() {
    log_info!("test wtr_wizard_error_apply_change_mode_wizard started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_wizard finished");
}

#[test]
fn wtr_wizard_error_apply_change_mode_wizard_with_std_cycles_000_037() {
    log_info!("test wtr_wizard_error_apply_change_mode_wizard_with_std_cycles started");

    let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state_with_std_cycles(Mode::Wizard, State::WzrWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_apply_change_mode_wizard_with_std_cycles finished");
}

// deve ir para shutdown
#[test]
fn wtr_all_modes_error_apply_shutdown_000_038() {
    log_info!("test wtr_all_modes_error_apply_shutdown_000_045 started");
    const MODES: [Mode; 3] = [Mode::Manual, Mode::Standard, Mode::Wizard];
    const TRANSITION_STATES: [State; 3] = [State::ManWait, State::NoScheduleDef, State::WzrWait];

    for i in 0..3 {
        let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(MODES[i], TRANSITION_STATES[i]);
        time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ShutDown, State::Shutdown);
        terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    }
    log_info!("test wtr_all_modes_error_apply_shutdown_000_045 finished");
}

// deve manter-se em error
#[test]
fn wtr_all_modes_error_apply_error_000_039() {
    log_info!("test wtr_standard_error_apply_error started");
    const MODES: [Mode; 3] = [Mode::Manual, Mode::Standard, Mode::Wizard];
    const TRANSITION_STATES: [State; 3] = [State::ManWait, State::NoScheduleDef, State::WzrWait];
    for i in 0..3 {
        let (msg_broker, handle_evt_mng, mut wtr_svc, mut time_tick) = explore_error_state(MODES[i], TRANSITION_STATES[i]);
        time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);
        terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    }
    log_info!("test wtr_standard_error_apply_error finished");
}

#[test]
fn wtr_all_modes_shutdown_try_cmd_with_no_effect_000_040() {
    log_info!("test wtr_all_modes_shutdown_try_cmd_with_no_effect_000_051 started");

    const MODES: [Mode; 3] = [Mode::Manual, Mode::Standard, Mode::Wizard];
    const TRANSITION_STATES: [State; 3] = [State::ManWait, State::NoScheduleDef, State::WzrWait];

    for i in 0..3 {
        let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
        let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

        let (db, start_up) = prepare_common(time_ref, MODES[i], prev_water);

        //criamos a maquina um pouco antes das 22 do dia 2 para calcular o tempo do ciclo para o dia certo
        let time_tick = time_ref.add_secs(1);
        let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

        cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
        assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclo standard definido");
        assert_start_params(db.clone(), MODES[i], &wtr_svc, start_up.clone(), TRANSITION_STATES[i]);

        let mut time_tick = time_ref.add_secs(1);

        time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ShutDown, State::Shutdown);

        let cmds = build_cmds([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], MODES[i]);
        validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::Shutdown, &cmds);

        terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    }
    log_info!("test wtr_all_modes_shutdown_try_cmd_with_no_effect_000_051 finished");
}

/// 1. Command::ChangeMode(mode), <br>
/// 2. Command::ShutDown,<br>
/// 3. Command::Error,<br>
/// 4. Command::ChangeState,<br>
/// 5. Command::StartCycle(0),<br>
/// 6. Command::Start,<br>
/// 7. Command::Null,<br>
/// 8. Command::StartSector,<br>
/// 9. Command::EndCycle(0),<br>
/// 10. Command::ForceSector(0),<br>
/// 11. Command::Suspend(Alert::new(AlertType::WIND, 20.)),<br>
/// 12. Command::Resume,<br>
/// 13. Command::ResumeTimeOut,<br>
/// 14. Command::StopCycle(cycle_id),<br>
/// 15. Command::ForceCycle(cycle_id),<br>
/// 16. Command::StopSector(running_ptr.clone()),<br>
/// 17. Command::EndSector(running_ptr),<br>
fn build_cmds(idx: [u8; 17], mode: Mode) -> Vec<Command> {
    let cycle_id = 0; //Isto é dummy - não é suposto ser usado para nada
    let running_ptr = RunningPtr { cycle: Some(0), sec_id: Some(0), run_sec_ptr: Some(0) };

    let cmds = [
        Command::ChangeMode(mode),
        Command::ShutDown,
        Command::Error,
        Command::ChangeState,
        Command::StartCycle(0),
        Command::Start,
        Command::Null,
        Command::StartSector,
        Command::EndCycle(0),
        Command::ForceSector(0),
        Command::Suspend(Alert::new(AlertType::WIND, 20.)),
        Command::Resume,
        Command::ResumeTimeOut,
        Command::StopCycle(cycle_id),
        Command::ForceCycle(cycle_id),
        Command::StopSector(running_ptr.clone()),
        Command::EndSector(running_ptr),
    ];

    let x = cmds[..].iter().zip(idx).filter(|pair| pair.1 == 1).map(|p| p.0.clone()).collect();
    x
}

#[test]
fn wtr_standard_try_cmd_with_no_effect_when_in_wait_000_041() {
    log_info!("test standard_try_cmd_with_no_effect_when_not_running started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let mut time_tick = time_ref.add_secs(1);
    // water acc á mais de 7 dias = 0
    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::ChangeState;
    // cmd = Command::Error;
    // no std wait este funciona...mas só deve funcionar se estiver dentro dos critérios do timing - testar noutro cenário
    // cmd = Command::StartCycle(0);
    let cmds = build_cmds([0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Standard);

    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::StdWait, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test standard_try_cmd_with_no_effect_when_not_running finished");
}

fn validate_list_cmd_nop(wtr_svc: &mut WtrSvc, time_tick: &mut CtrlTime, test_state: State, cmds: &[Command]) {
    for cmd in cmds {
        validate_cmd_nop(wtr_svc, time_tick, cmd.clone(), test_state)
    }
}

fn validate_cmd_nop(wtr_svc: &mut WtrSvc, time_tick: &mut CtrlTime, cmd: Command, test_state: State) {
    wtr_svc.snd_command(cmd.clone());
    *time_tick = time_tick.add_secs(1);
    wtr_svc.verify_things_to_do(*time_tick);
    assert!(wtr_svc.engine.wtr_cfg.state == test_state, "failled for command: {}", cmd);
}

#[test]
fn wtr_wizard_try_cmd_with_no_effect_when_in_wait_000_042() {
    log_info!("test wtr_wizard_try_cmd_with_no_effect_when_in_wait started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Wizard);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let mut time_tick = time_ref.add_secs(1);
    // water acc á mais de 7 dias = 0
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = Command::ChangeMode(MODE::STANDARD);
    // cmd = Command::ShutDown;
    // cmd = Command::ChangeState;
    // cmd = Command::Error;
    // no std wait este funciona...mas só deve funcionar se estiver dentro dos critérios do timing - testar noutro cenário
    // cmd = Command::StartCycle(0);
    let cmds = build_cmds([0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Mode::Wizard);

    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::WzrWait, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_try_cmd_with_no_effect_when_in_wait finished");
}

#[test]
fn wtr_manual_try_cmd_with_no_effect_when_in_wait_000_043() {
    log_info!("test wtr_manual_try_cmd_with_no_effect_when_in_wait started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let mut time_tick = time_ref.add_secs(1);
    // water acc á mais de 7 dias = 0
    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd = ChangeMode(MODE)
    //        ShutDown
    //        Error
    //        ForceCycle
    //        ForceSector
    //        ChangeState
    let cmds = build_cmds([0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1], Mode::Manual);

    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::ManWait, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_manual_try_cmd_with_no_effect_when_in_wait finished");
}

// simulamos o arranque da rega no modo standard
// a máquina nunca permanece no modo cycle wtr, porque faz auto drive para o sec wtr, oue stado wait
#[test]
fn wtr_standard_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_000_044() {
    log_info!("test wtr_standard_try_cmd_with_no_effect_when_in_cycle_wtr started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Standard);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    assert_eq!(&prev_water.add_days(22).as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);
    establish_sectors_base_condition_before_cycle_start(&mut wtr_svc);

    //validar o estado inicial dos setores
    let mut time_tick = new_start;
    {
        //em tese isto arranca a rega
        process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> processo de arranque + cycle start -> one schedule -> all sectors"));
        // e aqui é que os tempos dos setores devem estar certos
        validate_sec_times(new_start, &wtr_svc);

        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
        println!("ciclo 0 ativo - fora do loop");

        // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
        // cmd = ChangeMode(MODE)
        //      ShutDown
        //      Error
        //      EndSector
        //      StopCycle;
        //                                       1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17
        let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0], Mode::Standard);

        validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::StdWtrSector, &cmds);
    }
    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_standard_try_cmd_with_no_effect_when_in_cycle_wtr finished");
}

#[test]
fn wtr_wzrd_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_000_045() {
    log_info!("test wtr_wzrd_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Wizard);

    // vamos apagar os sensores a ver se não se influencia o tempo da rega.
    _ = wtr_svc.engine.db.delete_daily_measures();

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap() as usize;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    establish_sectors_base_condition_before_cycle_start(&mut wtr_svc);

    // assert_sectors_base_condition_before_cycle_start(&wtr_svc);
    for i in 0..MAX_SECTORS {
        let sec = &mut wtr_svc.engine.sectors[i];
        sec.deficit = 150.;
    }
    //validar o estado inicial dos setores
    let mut time_tick = new_start;
    {
        //em tese isto arranca a rega
        process_tick(&mut wtr_svc, time_tick, &String::from("mode wizard -> processo de arranque + cycle start -> one schedule -> first sector"));
        //estabelecer as condições iniciais para facilitar as validações
        let time_aux = new_start.clone();
        // eu aqui fiz as contas á mão, para estas condições de arranque.
        let oraculo_tempos_wizard: [f32; MAX_SECTORS] = [24., 20., 30., 30., 30., 30.];
        let mut next: u64 = 0;
        for i in 0..MAX_SECTORS {
            let rs = &wtr_svc.engine.run_secs[i];
            assert!(
                (rs.wtr_tgt_min - oraculo_tempos_wizard[i]).abs() < f32::EPSILON,
                "target de minutos para a rega - setor {} - left: {}, right: {}",
                i,
                rs.wtr_tgt_min,
                oraculo_tempos_wizard[i]
            );
            if i < 1 {
                assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", i);
            } else {
                assert_eq!(rs.status, WateringStatus::Waiting, "estado da rega do setor {}", i);
            }
            assert!((rs.start.0 as f64 - (time_aux + next).0 as f64).abs() < GIGA_F, "hora de inicio do setor",);
            assert!((rs.end.0 as f64 - (rs.start + min_to_nano(rs.wtr_tgt_min)).0 as f64).abs() < GIGA_F, "hora de fim do setor",);
            next = next + (oraculo_tempos_wizard[i] * 60. * GIGA_F as f32) as u64 + wtr_svc.engine.wtr_cfg.pump_recycle_time as u64 * GIGA_U;
        }

        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
        println!("ciclo 0 ativo - fora do loop");

        // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
        // cmd =ChangeMode(MODE)
        //       ShutDown
        //       Error
        //       EndSector
        //       StopCycle
        //       Suspend
        //                                                          1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17
        let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0], Mode::Wizard);

        validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::WzrWtrSector, &cmds);
    }
    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wzrd_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr finished");
}

#[test]
fn wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_cycle_000_046() {
    log_info!("test wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_cycle started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    // houve uma alteração nos requisitos e passei a só considerar aceites forcde cycle o sector nos direct cycles
    let cycle_ptr = wtr_svc.engine.internal.direct.unwrap() as usize;
    let cycle_id = wtr_svc.engine.cycles[cycle_ptr].run.cycle_id;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);
    establish_secs_data_on_start(&mut wtr_svc);

    //validar o estado inicial dos setores
    let mut time_tick = new_start;

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ForceCycle(cycle_id), State::ManWtrSector);

    // validar os tempos dos setores devem estar certos
    validate_sec_times(new_start, &wtr_svc);

    let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
    assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo 0 ativo - fora do loop");

    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd =ChangeMode(MODE)
    //     ShutDown
    //     Error
    //     EndSector
    //     StopSector
    //     StopCycle
    //                                       1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0], Mode::Manual);
    validate_list_cmd_nop(&mut wtr_svc, &mut time_tick, State::ManWtrSector, &cmds);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_cycle finished");
}

fn validate_sec_times(new_start: CtrlTime, wtr_svc: &WtrSvc) {
    //estabelecer as condições iniciais para facilitar as validações
    let time_aux = new_start.clone();
    let mut next: u64 = 0;
    for i in 0..MAX_SECTORS {
        let sec = &wtr_svc.engine.sectors[i];
        let rs = &wtr_svc.engine.run_secs[i];
        assert!(
            (rs.wtr_tgt_min - sec.max_duration).abs() < f32::EPSILON,
            "target de minutos para a rega - setor {} - left: {}, right: {}",
            &sec.name,
            rs.wtr_tgt_min,
            sec.max_duration
        );
        if i < 1 {
            assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", i);
        } else {
            assert_eq!(rs.status, WateringStatus::Waiting, "estado da rega do setor {}", i);
        }
        // toleramos uma diferença de 6 segundos (porque aparentemente o erro acumula 1 seg entre setores)
        assert!(
            (rs.start.0 as f64 - (time_aux + next).0 as f64).abs() <= GIGA_F * 6.,
            "hora de inicio do setor {}.  É {} e devia ser {} ",
            i,
            rs.start.as_rfc3339_str_e(),
            (time_aux + next).as_rfc3339_str_e()
        );
        assert!(rs.end.0 as f64 - ((rs.start + min_to_nano(rs.wtr_tgt_min)).0 as f64).abs() < GIGA_F, "hora de fim do setor",);
        next = next + (sec.max_duration * 60. * GIGA_F32) as u64 + wtr_svc.engine.wtr_cfg.pump_recycle_time as u64 * GIGA_U;
    }
}

#[test]
fn wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_sec_000_047() {
    log_info!("test wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_sec started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    let time_tick = force_sector(&mut wtr_svc, time_ref, db, start_up, 0, validate_list_cmd_nop_man_sec_direct);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_man_try_cmd_with_no_effect_when_in_cycle_and_sec_wtr_forced_sec finished");
}

fn force_sector(
    wtr_svc: &mut WtrSvc, time_ref: CtrlTime, db: Persist, start_up: StartupData, sector_id: SECTOR_ID,
    some_test_func: fn(wtr_svc: &mut WtrSvc, time_tick: &mut CtrlTime),
) -> CtrlTime {
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(wtr_svc, &[true, true, true, true, true, true], time_ref.sub_days(1));
    let cycle_ptr = wtr_svc.engine.internal.direct.unwrap() as usize;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");
    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Manual, &*wtr_svc, start_up.clone(), State::ManWait);
    establish_secs_data_on_start(wtr_svc);

    let mut time_tick = new_start;

    //em tese isto arranca a rega
    time_tick = send_and_apply_command(wtr_svc, time_tick, Command::ForceSector(sector_id), State::ManWtrSectorDirect);

    // e aqui é que os tempos dos setores devem estar certos
    let time_aux = new_start.clone();
    let sec = &wtr_svc.engine.sectors[sector_id as usize];
    // no sec direct só há sempre um setor ativo
    let rs = &wtr_svc.engine.run_secs[0];

    let len_lista = wtr_svc.engine.run_secs.len();
    assert!(len_lista == 1, "nr de setores a regar é: {}", len_lista);
    assert!(
        (rs.wtr_tgt_min - Sector::MAX_SECTOR_WORK_MINUTES).abs() < f32::EPSILON,
        "target de minutos para a rega - setor {} - left: {}, right: {}",
        sector_id,
        rs.wtr_tgt_min,
        sec.max_duration
    );
    assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", sector_id);
    println!("start é: {} e devia ser: {}", rs.start.as_rfc3339_str_e(), time_aux.as_rfc3339_str_e());
    println!("end é: {} e devia ser: {}", rs.end.as_rfc3339_str_e(), (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e());

    assert!(rs.start.0 - time_aux.0 <= 1_000_000_000, "hora de inicio do setor",);
    assert_eq!(rs.end.as_rfc3339_str_e(), (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e(), "hora de fim do setor",);

    let ptr = wtr_svc.engine.active_ptrs.cycle.unwrap();
    assert_eq!(ptr, cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo direto {} ativo ", ptr);
    assert_eq!(sector_id, wtr_svc.engine.active_ptrs.sec_id.unwrap());
    println!("setor direto {} ativo ", wtr_svc.engine.active_ptrs.sec_id.unwrap());

    some_test_func(wtr_svc, &mut time_tick);

    // avançamos a máquina um pouco...
    let max_time = CtrlTime(CtrlTime::MAX - 1);
    process_tick(wtr_svc, max_time, &String::from("mode manual -> forced sector -> time tick -> -> do nothing -> first sector"));
    assert!(wtr_svc.engine.wtr_cfg.state == State::ManWtrSectorDirect);

    time_tick = time_tick.add_secs(3600);
    let running_ptr = RunningPtr { cycle: None, sec_id: None, run_sec_ptr: None };
    time_tick = send_and_apply_command(wtr_svc, time_tick, Command::StopSector(running_ptr), State::ManWait);
    thread::sleep(Duration::from_millis(100));
    for i in 0..MAX_SECTORS {
        let sec = &mut wtr_svc.engine.sectors[i];
        assert!(sec.state == RelayState::Closed);
    }
    time_tick
}

fn validate_list_cmd_nop_man_sec_direct(wtr_svc: &mut WtrSvc, time_tick: &mut CtrlTime) {
    // estes comandos funcionam em todos os estados, pelo que vão ser testados em separado para cada um dos estados
    // cmd =ChangeMode(MODE)
    //     ShutDown
    //     Error
    //     StopSector
    //                                       1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17
    let cmds = build_cmds([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1], Mode::Manual);
    validate_list_cmd_nop(wtr_svc, time_tick, State::ManWtrSectorDirect, &cmds);
}

fn establish_secs_data_on_start(wtr_svc: &mut WtrSvc) {
    for i in 0..MAX_SECTORS {
        let sec = &mut wtr_svc.engine.sectors[i];
        sec.deficit = 0.;
        sec.last_watered_in = CtrlTime(0);
        sec.last_change = CtrlTime(0);
    }
}

// não deve ser possivel um segundo ciclo entrar quanto está um primeiro ciclo ativo
// isto porque so o modo manual reage aos comandos "forced"
// isto como está corre bem, mas não sei se estou a testar tudo o que devia :-)
#[test]
fn wtr_start_standard_overpositioned_cycles_000_048() {
    log_info!("test wtr_start_standard_overpositioned_cycles started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Standard, prev_water);

    insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);
    // ciclo em sobreposição com o primeiro
    insert_schedule(prev_water.add_secs(30), &db, String::from("standard 2"), CycleType::Standard);

    //criamos a maquina um pouco antes das 22 do dia seguinte ao arranque para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.std_ptrs.len(), 2, "dois ciclos standard existente");
    let cycle_ptr1 = wtr_svc.engine.std_ptrs[0].1 as usize;
    let cycle_ptr2 = wtr_svc.engine.std_ptrs[1].1 as usize;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 2, "cinco ciclos no total");

    let new_start1 = wtr_svc.engine.cycles[cycle_ptr1].schedule.start;
    let new_start2 = wtr_svc.engine.cycles[cycle_ptr2].schedule.start;

    let next_calc_wtr1 = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr1].schedule).unwrap().unwrap();
    let next_calc_wtr2 = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr2].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr1.as_rfc3339_str_e(), &new_start1.as_rfc3339_str_e(), "data de arranque do ciclo 1");
    assert_eq!(&next_calc_wtr2.as_rfc3339_str_e(), &new_start2.as_rfc3339_str_e(), "data de arranque do ciclo 2");

    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);

    establish_secs_data_on_start(&mut wtr_svc);

    //arrancamos o primeiro ciclo
    let mut time_tick = new_start1;
    process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> start + cycle start -> two schedules -> first cycle + first sector"));
    assert!(wtr_svc.engine.wtr_cfg.state == State::StdWtrSector);

    assert_eq!(wtr_svc.engine.active_ptrs.cycle.unwrap() as usize, cycle_ptr1, " Ciclo 1 a regar");
    assert_eq!(wtr_svc.engine.active_ptrs.sec_id.unwrap() as usize, 0, " setor 1 a regar");
    // e agora com o primeiro setor a regar, vai chegar o tempo do 2º ciclo
    time_tick = new_start2;
    process_tick(&mut wtr_svc, time_tick, &String::from("mode manual -> start + cycle start -> two schedules -> try second cycle"));

    // o estado deve manter-se, mas também se deve manter o primeiro ciclo a regar
    assert!(wtr_svc.engine.wtr_cfg.state == State::StdWtrSector);
    // e deve-se manter o 1º ciclo a regar
    assert_eq!(wtr_svc.engine.active_ptrs.cycle.unwrap() as usize, cycle_ptr1, " Ciclo 1 a regar");
    assert_eq!(wtr_svc.engine.active_ptrs.sec_id.unwrap() as usize, 0, " setor 1 a regar");

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_standard_overpositioned_cycles finished");
}

//--------------  MODE MANUAL

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo
#[test]
fn wtr_start_manual_force_cycle_not_runing_000_049() {
    log_info!("test wtr_start_manual_force_cycle_not_runing started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    let cycle_ptr = wtr_svc.engine.internal.direct.unwrap() as usize;
    let cycle_id = wtr_svc.engine.cycles[cycle_ptr].run.cycle_id;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);
    establish_secs_data_on_start(&mut wtr_svc);

    //validar o estado inicial dos setores
    let mut time_tick = new_start;

    //em tese isto arranca a rega
    wtr_svc.snd_command(Command::ForceCycle(cycle_id));
    process_tick(&mut wtr_svc, time_tick, &String::from("mode manual -> start + forced cycle start -> one schedule -> first sector"));

    assert!(wtr_svc.engine.wtr_cfg.state == State::ManWtrSector);
    // validar os tempos dos setores devem estar certos

    validate_sec_times(new_start, &wtr_svc);

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, &Vec::<bool>::new(), cycle_ptr, State::ManWait, Mode::Manual, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_manual_force_cycle_not_runing finished");
}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo, parando o ciclo em execução
// Este cenário deixou de ser possivel.  O interface só permite forçar ciclos ou setores mas sempre no ciclo "direct"
// #[test]
// fn wtr_start_manual_force_cycle_runing_000_050() {
//     log_info!("test wtr_start_manual_force_cycle_runing started");
//     let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
//     let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
//     let mode = Mode::Manual;
//     let (db, start_up) = prepare_common(time_ref, mode, prev_water);

//     insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);
//     insert_schedule(prev_water.sub_days(1), &db, String::from("standard 2"), CycleType::Standard);

//     //criamos a maquina um pouco antes das 22 do dia seguinte ao arranque para calcular o tempo do ciclo para o dia certo
//     let time_tick = time_ref.add_secs(85000);
//     let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

//     //vamos colocar todos os setores desejados disabled
//     cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

//     let cycle_ptr = get_nth_standard_cycle(&wtr_svc, 1).unwrap();
//     let cycle_ptr2 = get_nth_standard_cycle(&wtr_svc, 2).unwrap();

//     let cycle_id = wtr_svc.engine.cycles[cycle_ptr].run.cycle_id;
//     let cycle_id2 = wtr_svc.engine.cycles[cycle_ptr2].run.cycle_id;

//     assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 2, "dois ciclos standard existente");

//     let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

//     let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
//     assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

//     assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

//     establish_secs_data_on_start(&mut wtr_svc);

//     //validar o estado inicial dos setores
//     let mut time_tick = new_start;

//     let t0: Instant;
//     //em tese isto arranca a rega
//     time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ForceCycle(cycle_id), State::ManWtrSector);

//     // validar os tempos dos setores devem estar certos

//     validate_sec_times(new_start, &wtr_svc);

//     let nr_skipped: usize = 0;

//     if nr_skipped < MAX_SECTORS {
//         let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
//         assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo standard ativo");
//         println!("ciclo {} ativo - fora do loop", cycle_ptr);

//         assert_cycle_start_and_first_sector_running_enhanced(time_tick, &wtr_svc, 0);
//     }
//     // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
//     let max_run_secs = MAX_SECTORS - nr_skipped;
//     let mut run_sec = 0;
//     let mut i = 0;
//     let stop_at_sec_nr = 3;
//     assert!(stop_at_sec_nr <= MAX_SECTORS);
//     while i < MAX_SECTORS {
//         if i < max_run_secs {
//             // sabemos que estão 6 setores na BD
//             // no primeiro já ficou ativado .com o time_tick de fora do ciclo, ou com o ultimo time_tick do loop
//             assert_active_sec_open(&wtr_svc, i + nr_skipped);
//             let run_sec_id = wtr_svc.engine.active_ptrs.run_sec_ptr.unwrap() as usize;
//             time_tick = advance_to_end_of_sector(&wtr_svc, run_sec_id, time_tick);
//             // e agora aqui com um setor ativo, vamos forçar a execução de outro ciclo
//             // é suposto parar tudo do ciclo 1 e arrancar do ciclo 2
//             if i == stop_at_sec_nr {
//                 // t0 = Instant::now();
//                 let cycle_id = wtr_svc.engine.cycles[cycle_ptr].run.cycle_id;
//                 t0 = Instant::now();
//                 wtr_svc.snd_command(Command::StopCycle(cycle_id as u32));
//                 println!("snd command -> stop cycle: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));

//                 time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ForceCycle(cycle_id2), State::ManWtrSector);
//                 i += 1;
//                 // validamos que os restantes setores do ciclo 1 estão desligados
//                 for j in i..MAX_SECTORS {
//                     assert_valve_closed(cycle_ptr as u8, j, &wtr_svc);
//                 }
//                 break;
//             } else {
//                 // nos setores antes de chegar ao setor da interrupção vamos avançando, ou seja, parando
//                 let rs = &wtr_svc.engine.run_secs[run_sec];
//                 time_tick = rs.end;
//                 process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> sector stop"));
//                 assert_valve_closed(cycle_ptr as u8, run_sec, &wtr_svc);

//                 if i < max_run_secs - 1 {
//                     // arrancamos o setor seguinte
//                     time_tick = wtr_svc.engine.run_secs[run_sec + 1].start;
//                     process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> sector start"));
//                 }
//             }
//         }
//         run_sec += 1;
//         i += 1;
//     }
//     // e agora testam,os o 2º ciclo
//     // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
//     run_sec = 0;
//     i = 0;

//     while i < MAX_SECTORS {
//         // sabemos que estão 6 setores na BD
//         // no primeiro já ficou ativado .com o time_tick de fora do ciclo, ou com o ultimo time_tick do loop
//         let _run_sec_id = wtr_svc.engine.active_ptrs.run_sec_ptr.as_ref().unwrap().clone() as usize;
//         let sec_id = assert_active_sec_open(&wtr_svc, i + nr_skipped);
//         let sec = &wtr_svc.engine.run_secs[_run_sec_id];
//         time_tick = sec.end;
//         //saltamos para o fim
//         // tempo para parar o setor - setor ainda ativo
//         process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> sector stop"));
//         assert_sec_closed(&wtr_svc, sec_id);
//         if i < max_run_secs - 1 {
//             time_tick = wtr_svc.engine.run_secs[run_sec + 1].start;
//             process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> sector start"));
//         }
//         run_sec += 1;
//         i += 1;
//     }
//     terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
//     log_info!("test wtr_start_manual_force_cycle_runing finished");
// }

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo
// sectors overwatered não terão influência
// o teste 59, 60, 62, 63 já exercitam isto, sendo que o nivel de watered no modo manual não têm influência.
// #[test]
// fn wtr_start_manual_force_cycle_sectors_over_watered_000_064() {}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo
// e força um erro - para ver a maquina a mudar para o estado error
#[test]
fn wtr_start_manual_force_cycle_sectors_error_happens_000_051() {
    log_info!("test wtr_start_manual_force_cycle_sectors_error_happens_000_065 started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let mode = Mode::Manual;
    let (db, start_up) = prepare_common(time_ref, mode, prev_water);

    insert_schedule(prev_water, &db, String::from("standard 1"), CycleType::Standard);

    //criamos a maquina um pouco antes das 22 do dia seguinte ao arranque para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref.add_secs(85000);
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    let cycle_ptr = wtr_svc.engine.internal.direct.unwrap() as usize;
    let cycle_id = wtr_svc.engine.cycles[cycle_ptr].run.cycle_id;

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;

    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);
    establish_secs_data_on_start(&mut wtr_svc);

    //validar o estado inicial dos setores
    let mut time_tick = new_start;

    //em tese isto arranca a rega
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ForceCycle(cycle_id), State::ManWtrSector);

    // validar os tempos dos setores devem estar certos
    validate_sec_times(new_start, &wtr_svc);

    let nr_skipped: usize = 0;

    if nr_skipped < MAX_SECTORS {
        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo standard ativo");
        println!("ciclo {} ativo - fora do loop", cycle_ptr);
        assert_cycle_start_and_first_sector_running_enhanced(time_tick, &wtr_svc, 0);
    }
    // a agora avançar a máquina para ir mudando o estado da maquina e ir validando se os setores estão a abrir e a fechar e etc.
    let max_run_secs = MAX_SECTORS - nr_skipped;
    let mut run_sec = 0;
    let mut i = 0;
    let stop_at_sec_nr = 3;
    assert!(stop_at_sec_nr <= MAX_SECTORS);
    while i < MAX_SECTORS {
        if i < max_run_secs {
            // sabemos que estão 6 setores na BD
            // no primeiro já ficou ativado .com o time_tick de fora do ciclo, ou com o ultimo time_tick do loop
            assert_active_sec_open(&wtr_svc, i + nr_skipped);
            let run_sec_id = wtr_svc.engine.active_ptrs.run_sec_ptr.unwrap() as usize;
            time_tick = advance_to_end_of_sector(&wtr_svc, run_sec_id, time_tick);
            // e agora aqui com um setor ativo, vamos forçar a execução de outro ciclo
            // é suposto parar tudo do ciclo 1 e arrancar do ciclo 2
            if i == stop_at_sec_nr {
                time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::Error, State::Error);
                i += 1;
                // validamos que os restantes setores do ciclo 1 estão desligados
                for j in i..MAX_SECTORS {
                    assert_valve_closed(cycle_ptr as CYCLE_PTR, j, &wtr_svc);
                }
                break;
            } else {
                // nos setores antes de chegar ao setor da interrupção vamos avançando, ou seja, parando
                let rs = &wtr_svc.engine.run_secs[run_sec];
                time_tick = rs.end;
                process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> end sector -> one schedule -> sector stop"));
                assert_valve_closed(cycle_ptr as CYCLE_PTR, run_sec, &wtr_svc);
                if i < max_run_secs - 1 {
                    // arrancamos o setor seguinte
                    time_tick = wtr_svc.engine.run_secs[run_sec + 1].start;
                    process_tick(&mut wtr_svc, time_tick, &String::from("mode standard -> sector start -> one schedule -> sector start"));
                }
            }
        }
        run_sec += 1;
        i += 1;
    }

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_manual_force_cycle_sectors_error_happens_000_065 finished");
}

fn assert_valve_closed(cycle_ptr: CYCLE_PTR, run_sec_id: usize, wtr_svc: &WtrSvc) {
    let rs = &wtr_svc.engine.run_secs[run_sec_id];
    let valve_state = wtr_svc.engine.sectors[rs.sec_id as usize].state.clone();
    assert!(valve_state == RelayState::Closed, "setor {} do ciclo {} está parado", rs.sec_id, cycle_ptr);
}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo
// e força um erro - para ver a maquina a mudar para o estado error e a ignorar os proximos ciclos
// isto já é testdo lá atrás
// #[test]
// fn wtr_start_manual_force_cycle_sectors_error_happens_next_cycle_ignored_000_066() {}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução de um ciclo
// e força um erro - para ver a maquina a mudar para o estado error, a ignorar ciclos, e depois de colocar em manual again, limpa o erro e já executa
// também já testado lá atrás
// #[test]
// fn wtr_start_manual_force_cycle_sectors_error_happens_and_recover_000_067() {}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução do setor n
#[test]
fn wtr_start_manual_force_sector_n_000_052() {
    log_info!("test wtr_start_manual_force_sector_n_000_068 started");

    for sec in 0..MAX_SECTORS {
        let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
        let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
        let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

        let time_tick = force_sector(&mut wtr_svc, time_ref, db, start_up, sec as u8, no_op_func);

        terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    }
    log_info!("test wtr_start_manual_force_sector_n_000_068 finished");
}

fn no_op_func(_wtr_svc: &mut WtrSvc, _time_tick: &mut CtrlTime) {}

fn force_sector_with_error(
    wtr_svc: &mut WtrSvc, time_ref: CtrlTime, db: Persist, start_up: StartupData, sector_id: SECTOR_ID,
    some_test_func: fn(wtr_svc: &mut WtrSvc, time_tick: &mut CtrlTime), recover: bool,
) -> CtrlTime {
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(wtr_svc, &[true, true, true, true, true, true], time_ref.sub_days(1));
    let cycle_ptr = wtr_svc.engine.internal.direct.unwrap() as usize;
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");
    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    let next_calc_wtr = find_next_event(time_ref, &wtr_svc.engine.cycles[cycle_ptr].schedule).unwrap().unwrap();
    assert_eq!(&next_calc_wtr.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Manual, &*wtr_svc, start_up.clone(), State::ManWait);
    establish_secs_data_on_start(wtr_svc);

    let mut time_tick = new_start;

    //em tese isto arranca a rega
    time_tick = send_and_apply_command(wtr_svc, time_tick, Command::ForceSector(sector_id), State::ManWtrSectorDirect);

    // e aqui é que os tempos dos setores devem estar certos
    let time_aux = new_start.clone();
    let sec = &wtr_svc.engine.sectors[sector_id as usize];
    // no sec direct só há sempre um setor ativo
    let rs = &wtr_svc.engine.run_secs[0];

    let len_lista = wtr_svc.engine.run_secs.len();
    assert!(len_lista == 1, "nr de setores a regar é: {}", len_lista);
    assert!(
        (rs.wtr_tgt_min - Sector::MAX_SECTOR_WORK_MINUTES).abs() < f32::EPSILON,
        "target de minutos para a rega - setor {} - left: {}, right: {}",
        sector_id,
        rs.wtr_tgt_min,
        sec.max_duration
    );
    assert_eq!(rs.status, WateringStatus::Running, "estado da rega do setor {}", sector_id);
    println!("start é: {} e devia ser: {}", rs.start.as_rfc3339_str_e(), time_aux.as_rfc3339_str_e());
    println!("end é: {} e devia ser: {}", rs.end.as_rfc3339_str_e(), (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e());

    assert!(rs.start.0 - time_aux.0 <= 1_000_000_000, "hora de inicio do setor",);
    assert_eq!(rs.end.as_rfc3339_str_e(), (rs.start + min_to_nano(rs.wtr_tgt_min)).as_rfc3339_str_e(), "hora de fim do setor",);

    let ptr = wtr_svc.engine.active_ptrs.cycle.unwrap();
    assert_eq!(ptr, cycle_ptr as u8, "id do ciclo ativo");
    println!("ciclo direto {} ativo ", ptr);
    assert_eq!(sector_id, wtr_svc.engine.active_ptrs.sec_id.unwrap());
    println!("setor direto {} ativo ", wtr_svc.engine.active_ptrs.sec_id.unwrap());

    some_test_func(wtr_svc, &mut time_tick);

    // avançamos a máquina um pouco...
    let max_time = CtrlTime(CtrlTime::MAX - 1);
    process_tick(wtr_svc, max_time, &String::from("mode manual -> forced sector -> time tick -> -> do nothing -> first sector"));
    assert!(wtr_svc.engine.wtr_cfg.state == State::ManWtrSectorDirect);

    time_tick = send_and_apply_command(wtr_svc, time_tick, Command::Error, State::Error);

    assert_all_secs_closed(wtr_svc);

    time_tick = time_tick.add_secs(3600);
    if recover {
        time_tick = send_and_apply_command(wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);
    }
    thread::sleep(Duration::from_millis(100));
    time_tick
}

fn assert_all_secs_closed(wtr_svc: &mut WtrSvc) {
    for i in 0..MAX_SECTORS {
        let sec = &mut wtr_svc.engine.sectors[i];
        assert!(sec.state == RelayState::Closed);
    }
}

// ver o arranque com um ciclo cujas condições são manual
// e força a execução do setor n e um erro acontece
#[test]
fn wtr_start_manual_force_sector_n_with_error_000_053() {
    log_info!("test wtr_start_manual_force_sector_n_with_error_000_074 started");
    for sec in 0..MAX_SECTORS {
        let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
        let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
        let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

        let time_tick = force_sector_with_error(&mut wtr_svc, time_ref, db, start_up, sec as u8, no_op_func, false);

        terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    }
    log_info!("test wtr_start_manual_force_sector_n_with_error_000_074 finished");
}

#[test]
fn wtr_start_manual_force_sector_1_with_error_and_recover_000_054() {
    log_info!("test wtr_start_manual_force_sector_1_with_error_000_074 started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    let time_tick = force_sector_with_error(&mut wtr_svc, time_ref, db, start_up, 1, no_op_func, true);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_manual_force_sector_1_with_error_000_074 finished");
}

// // o ciclo começa e a meio chega um alerta - não deve acontecer nada, para além de informar o cliente das condições meteo
// já testado lá atrás// já testado lá atrás
// ver o arranque com um ciclo cujas condições são manual
// e força a execução do setor 1, durante a execução de outro ciclo.  pára o outro ciclo e executa o novo
// #[test]
// fn wtr_start_manual_force_sector_1_running_000_076() {}

// // o ciclo começa e a meio chega um alerta - não deve acontecer nada, para além de informar o cliente das condições meteo
// já testado lá atrás// já testado lá atrás

// #[test]
// fn wtr_start_manual_run_suspend_with_wind_alert_000_077() {}

// o ciclo começa e a meio chega um alerta - não deve acontecer nada, para além de informar o cliente das condições meteo
// já testado lá atrás
// #[test]
// fn wtr_start_manual_run_suspend_with_rain_alert_000_078() {}

// o ciclo começa e a meio chega um alerta - não deve acontecer nada, para além de informar o cliente das condições meteo
// já testado lá atrás
// #[test]
// fn wtr_start_manual_run_suspend_with_wind_and_rain_alert_000_079() {}

//---

//ver o arranque com um ciclo wizard onde ainda não chegou ao tempo para correr
// não deve acontecer nada
#[test]
fn wtr_start_wizard_schedule_before_time_000_055() {
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect started");

    // Domingo - dia 0 da semana
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 22, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, prev_water);

    //criamos a maquina um pouco antes das 21:50 do dia 222 para calcular o tempo do ciclo para o dia certo
    let time_tick = time_ref;
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "sem ciclos standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    let mut time_tick = time_ref.add_secs(1);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap();
    let cycle = &wtr_svc.engine.cycles[cycle_ptr as usize];
    let cycle_start = cycle.schedule.start;
    while time_tick < cycle_start {
        wtr_svc.verify_things_to_do(time_tick);
        time_tick = time_tick.add_secs(2);
        assert!(wtr_svc.engine.wtr_cfg.state == State::WzrWait, "state");
    }

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect finished");
}

fn wtr_start_standard_schedule_all_secs(start_time: CtrlTime, end_time: CtrlTime, step_secs: u64, prev_water: CtrlTime) {
    log_info!("test wtr_start_standrd_schedule_all_secs started");

    let (db, start_up, time_tick, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard_no_mangle(start_time, prev_water, Mode::Standard);

    //criamos a maquina um pouco antes das 21:50 do dia 22 para calcular o tempo do ciclo para o dia certo
    let mut time_tick = time_tick;

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), 3, "um ciclo standard");
    assert_start_params(db.clone(), Mode::Standard, &wtr_svc, start_up.clone(), State::StdWait);

    time_tick = time_tick.add_secs(1);

    let cycle_ptr = wtr_svc.engine.std_ptrs[0].1 as usize;
    let cycle = &wtr_svc.engine.cycles[cycle_ptr];
    let cycle_start = cycle.schedule.start;
    let mut total_cycles = 0;
    let mut tempo_total_em_rega = 0;
    let mut prev_start: CtrlTime = cycle_start;
    let mut prev_is_active = false;
    let mut actual_is_active: bool = false;
    let mut iter_counter: usize = 0;
    let mut next_start: CtrlTime;
    let mut d: DateTimeE;
    let mut stored_month = 1;
    let mut prev_state = wtr_svc.state(); 
    while time_tick < end_time {
        time_tick = time_tick.add_secs(step_secs);
        if time_tick >= wtr_svc.engine.cycles[cycle_ptr].schedule.start && !actual_is_active{
            println!("Vai arrancar o ciclo");
        }
        wtr_svc.verify_things_to_do(time_tick);
        next_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
        if wtr_svc.engine.active_ptrs.cycle.is_some() {
            actual_is_active = true;
        } else {
            actual_is_active = false;
        }
        if actual_is_active != prev_is_active {
            if actual_is_active {
                assert!(time_tick == next_start, "left: {}, right: {}", time_tick.as_rfc3339_str_e(), next_start.as_rfc3339_str_e());
                prev_start = time_tick;
                total_cycles += 1;
            }
            if !actual_is_active {
                tempo_total_em_rega += time_tick.0 - prev_start.0;
            }
            prev_is_active = actual_is_active;
        }
        if actual_is_active {
            if wtr_svc.engine.active_ptrs.sec_id.is_some() {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::StdWtrSector);
            } else {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::StdWtrCycle);
            }
            
        } else {
            assert_engine_state(&wtr_svc, iter_counter, time_tick, next_start, State::StdWait);
        }
        iter_counter += 1;
        d = time_tick.as_utc_date_time_e();
        if stored_month != d.month {
            println!("Estamos em {}/{}/{}", d.year, d.month, d.day);
            stored_month = d.month;
        }
        if wtr_svc.state() != prev_state{
            thread::sleep(Duration::from_millis(500));
            prev_state = wtr_svc.state();
        }
    }

    println!("Começou en:{}", start_time.as_rfc3339_str_e());
    println!("Nr total de ciclos: {}", total_cycles);
    println!("Tempo total em rega: {}", elapsed_dyn(tempo_total_em_rega));
    println!("Terminou em: {}", time_tick.as_rfc3339_str_e());

    unsafe { SHUTTING_DOWN = true };
    terminate_and_wait(wtr_svc, time_tick, msg_broker, handle_evt_mng);
    log_info!("test wtr_start_standrd_schedule_all_secs finished");
}

// e arranjando uma metrica para a corrente que se gasta no modo standard, e depois a corrente que se gasta no modo wizard (total das horas de rega) neste modo
// e em modo standard
// devia-se fazer isto no modo standard
// estes dados ficam na BD, pelo que é uma questão de depois ir ler os dados
// - mas isso implica não esquecer de corer estes testes isolados, para não apagar a bd no teste seguinte

// Nr total de ciclos: 19722
// Tempo total em rega: Total Time (days): 2488.076
// Isto levou das 16:28 as 20:30 portanto 4 horas
// é para correr durante a noite e para referência para o wizard
#[test] // só para correr para testes exaustivos
#[ignore]
fn run_all_from_start_to_end_of_universe_000_056() {
    let start_time = CtrlTime::from_utc_parts(1970, 01, 01, 1, 40, 0);
    let prev_water = CtrlTime::from_utc_parts(1970, 01, 01, 1, 30, 0);
    let end_time = CtrlTime::from_utc_parts(2077, 12, 31, 0, 0, 0);

    wtr_start_standard_schedule_all_secs(start_time, end_time, 1, prev_water);
}
  
#[test]
fn run_all_one_cycle_covered_000_057() {
    let start_time = CtrlTime::from_utc_parts(2022, 06, 24, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 06, 22, 0, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 06, 24, 7, 0, 0);

    wtr_start_standard_schedule_all_secs(start_time, end_time, 1, prev_water);
}

#[test] //só para correr de vez em quando
#[ignore]
fn run_all_six_months_covered_000_058() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 01, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2021, 12, 31, 0, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 06, 31, 0, 0, 0);

    wtr_start_standard_schedule_all_secs(start_time, end_time, 1, prev_water);
}

#[inline]
fn assert_engine_state(wtr_svc: &WtrSvc, iter_counter: usize, time_tick: CtrlTime, next_start: CtrlTime, state: State) {
    assert!(
        wtr_svc.engine.wtr_cfg.state == state,
        "state na iter: {} na data: {} para o ciclo que começou em: {}",
        iter_counter,
        time_tick.as_rfc3339_str_e(),
        next_start.as_rfc3339_str_e()
    )
}

// ver o arranque com um ciclo cujas condições são para arrancar
// e todos os ciclos são executados
#[test]
fn wtr_start_wizard_schedule_on_time_000_059() {
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect started");

    // Domingo 23 é o dia 0 da semana.
    // Em modo wizard como,  rega de 2 em 2 dias, teve que ter regado 2 dias antes, portanto na sexta 21
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, prev_water);

    //criamos a maquina um pouco antes das 21:50 do dia 22 para calcular o tempo do ciclo para o dia certo
    let mut time_tick = time_ref;
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclos standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    time_tick = time_ref.add_secs(1);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap();
    let cycle = &wtr_svc.engine.cycles[cycle_ptr as usize];
    let cycle_start = cycle.schedule.start;
    while time_tick < cycle_start {
        wtr_svc.verify_things_to_do(time_tick);
        time_tick = time_tick.add_secs(1);
        assert!(wtr_svc.engine.wtr_cfg.state == State::WzrWait, "state");
    }
    // E aqui deve arrancar a rega
    time_tick = time_tick.add_secs(1);
    wtr_svc.verify_things_to_do(time_tick);
    // REVIEW - é um bom teste, este final da semana
    // para ver como se comporta o acumulador de agua, no total previsto  ou afastado dele
    // cenários - final da semana no nivel previsto de week_acc - rega pelos tempos previstos
    //          - final da semana para além do nivel previsto   - A. ainda existe algo a adicionar para o nivel, e ai rega só pela diferença em falta
    //                                                          - B. Já não existe nada e ai salta todos os setores
    // e fazer o skip / diferentes niveis, sem setores, todos os setores, e as combinações possiveis dos setores
    // REVIEW o teste exaustivo disto implica correr pelo menos duas semanas, para ver a evolução do nivel, a mudança da semana
    // e já agora fazer um teste a correr para todos os anos desde a 1-1-1970 até 2077 para ver se apanhamos algum tema estranho algures
    // com injeção de erros, e recover, a ver o comportamento
    // simulando um periodo de inverno, primavera, verão e outono
    // e arranjando uma metrica para a corrente que se gasta no modo standard, e depois a corrente que se gasta no modo wizard (total das horas de rega) neste modo
    // e em modo standard
    // devia-se fazer isto no modo standard
    // estes dados ficam na BD, pelo que é uma questão de depois ir ler os dados
    // - mas isso implica não esquecer de correr estes testes isolados, para não apagar a bd no teste seguinte
    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, &Vec::<bool>::new(), cycle_ptr as usize, State::WzrWait, Mode::Wizard, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_wizard_error_try_cmd_with_no_effect finished");
}

// ver o arranque com um ciclo cujas condições para correr já passaram
// (isto não deve acontecer, mas caso aconteça, é para executar...desde que dentro do calendário de execução do ciclo)
// e todos os ciclos são executados
#[test]
fn wtr_start_wizard_schedule_after_time_000_060() {
    log_info!("test wtr_start_wizard_schedule_after_time_000_082 started");

    // Domingo 23 é o dia 0 da semana.
    // Em modo wizard como,  rega de 2 em 2 dias, teve que ter regado 2 dias antes, portanto na sexta 21
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, prev_water);

    //criamos a maquina um pouco antes das 21:50 do dia 22 para calcular o tempo do ciclo para o dia certo
    let mut time_tick = time_ref;
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclos standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap();
    let cycle = &wtr_svc.engine.cycles[cycle_ptr as usize];
    time_tick = cycle.schedule.start.add_secs(5);

    // E aqui deve arrancar a rega
    wtr_svc.verify_things_to_do(time_tick);

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, &Vec::<bool>::new(), cycle_ptr as usize, State::WzrWait, Mode::Wizard, 5);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_after_time_000_082 finished");
}

// ver o arranque com um ciclo cujas condições são para correr
// e todos os ciclos são para executar mas acontece um erro a meio
#[test]
fn wtr_start_wizard_schedule_with_error_000_061() {
    log_info!("test wtr_start_wizard_schedule_with_error_000_083 started");

    // Domingo 23 é o dia 0 da semana.
    // Em modo wizard como,  rega de 2 em 2 dias, teve que ter regado 2 dias antes, portanto na sexta 21
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up) = prepare_common(time_ref, Mode::Wizard, prev_water);

    //criamos a maquina um pouco antes das 21:50 do dia 22 para calcular o tempo do ciclo para o dia certo
    let mut time_tick = time_ref;
    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclos standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap();
    let cycle = &wtr_svc.engine.cycles[cycle_ptr as usize];
    time_tick = cycle.schedule.start.add_secs(5);

    // E aqui deve arrancar a rega
    wtr_svc.verify_things_to_do(time_tick);

    let secs_to_skip = &Vec::<bool>::new();
    // let expected_wait_state = State::WzrWait;
    let mode = Mode::Wizard;
    let disabled = get_disabled_positions(secs_to_skip);
    let nr_skipped = disabled.len();

    process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {mode} -> start + cycle start -> one schedule "), disabled.len());

    // se se saltarem todos os setores, não á nada a fazer
    if nr_skipped < MAX_SECTORS {
        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
        println!("ciclo {} ativo", cycle_ptr);

        assert_cycle_start_and_first_sector_running_enhanced(time_tick, &wtr_svc, 5);

        let nr_secs_to_wtr = wtr_svc.engine.run_secs.len();
        for run_sec_id in 0..nr_secs_to_wtr {
            let rs = &wtr_svc.engine.run_secs[run_sec_id];
            let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];

            let sec_id = assert_active_sec_open(&wtr_svc, sec.id as usize);

            if run_sec_id == 3 {
                wtr_svc.snd_command(Command::Error);
            }
            time_tick = advance_to_end_of_sector(&wtr_svc, run_sec_id, time_tick);

            process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {} -> sector end -> one schedule ", mode), nr_skipped);

            if run_sec_id >= 3 {
                assert!(wtr_svc.state() == State::Error, "Estado real {} esperado {}", wtr_svc.state(), State::Error);
                for run_sec_id in 3..nr_secs_to_wtr {
                    let rs = &wtr_svc.engine.run_secs[run_sec_id];
                    let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];
                    assert!(sec.state == RelayState::Closed, "Estado valvula real {} esperado {}", sec.state, RelayState::Closed);
                }
                break;
            }
            assert_sec_closed(&wtr_svc, sec_id);

            if run_sec_id < nr_secs_to_wtr - 1 {
                time_tick = wtr_svc.engine.run_secs[run_sec_id + 1].start;
                process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {} -> sector start -> one schedule ", mode), nr_skipped);
            }
        }
    }
    assert!(wtr_svc.engine.wtr_cfg.state == State::Error, "state");

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_with_error_000_083 finished");
}

// ver o arranque com um ciclo cujas condições são para correr
// e todos os ciclos são para executar mas acontece um erro a meio e depois recupera-se
#[test]
fn wtr_start_wizard_schedule_with_error_with_recover_000_062() {
    log_info!("test wtr_start_wizard_schedule_with_error_with_recover_000_084 started");

    // Domingo 23 é o dia 0 da semana.
    // Em modo wizard como,  rega de 1 em 1 dia, teve que ter regado 1 dias antes, portanto na madrugada do dia
    let mut time_tick = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 23, 05, 50, 0);

    let (db, start_up) = prepare_common(time_tick, Mode::Wizard, prev_water);

    let (msg_broker, handle_evt_mng, mut wtr_svc) = create_objects(&db, time_tick);

    prepare_wzrd_secs(&mut wtr_svc, &db, &start_up);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.deficit = 150.; // vamos dizer que é para regar o maximo permitido
    }

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap();
    let cycle = &wtr_svc.engine.cycles[cycle_ptr as usize];
    time_tick = cycle.schedule.start.add_secs(5);

    let old_start = cycle.schedule.start;

    // E aqui deve arrancar a rega
    wtr_svc.verify_things_to_do(time_tick);

    let secs_to_skip = &Vec::<bool>::new();
    let mode = Mode::Wizard;
    let disabled = get_disabled_positions(secs_to_skip);
    let nr_skipped = disabled.len();

    process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {mode} -> start + cycle start -> one schedule "), disabled.len());

    // se se saltarem todos os setores, não á nada a fazer
    if nr_skipped < MAX_SECTORS {
        let ptr = wtr_svc.engine.active_ptrs.cycle.as_ref().unwrap();
        assert_eq!(*ptr, cycle_ptr as u8, "id do ciclo ativo");
        println!("ciclo {} ativo", cycle_ptr);

        assert_cycle_start_and_first_sector_running_enhanced(time_tick, &wtr_svc, 5);

        let nr_secs_to_wtr = wtr_svc.engine.run_secs.len();
        for run_sec_id in 0..nr_secs_to_wtr {
            let rs = &wtr_svc.engine.run_secs[run_sec_id];
            let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];

            let sec_id = assert_active_sec_open(&wtr_svc, sec.id as usize);

            if run_sec_id == 3 {
                wtr_svc.snd_command(Command::Error);
            }
            time_tick = advance_to_end_of_sector(&wtr_svc, run_sec_id, time_tick);

            process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {} -> sector end -> one schedule ", mode), nr_skipped);

            if run_sec_id >= 3 {
                assert!(wtr_svc.state() == State::Error, "Estado real {} esperado {}", wtr_svc.state(), State::Error);
                for run_sec_id in 3..nr_secs_to_wtr {
                    let rs = &wtr_svc.engine.run_secs[run_sec_id];
                    let sec = &wtr_svc.engine.sectors[rs.sec_id as usize];
                    assert!(sec.state == RelayState::Closed, "Estado valvula real {} esperado {}", sec.state, RelayState::Closed);
                }
                break;
            }
            assert_sec_closed(&wtr_svc, sec_id);

            if run_sec_id < nr_secs_to_wtr - 1 {
                time_tick = wtr_svc.engine.run_secs[run_sec_id + 1].start;
                process_tick_in_proc(&mut wtr_svc, time_tick, &format!("mode {} -> sector start -> one schedule ", mode), nr_skipped);
            }
        }
    }
    assert!(wtr_svc.engine.wtr_cfg.state == State::Error, "machine state");

    // Agora tentamos a recuperação
    let t0 = Instant::now();
    wtr_svc.snd_command(Command::ChangeMode(Mode::Wizard));
    println!("send command change mode: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    //avançamos 1 hora
    //quer dizer que apanhamos a meio do ciclo, mas como o modo standard só valida o inicio do ciclo, vai fazer o reschedule do ciclo para o dia seguinte
    time_tick = time_tick.add_secs_f32(60.);
    println!("A máquina está no tempo: {}", time_tick.as_rfc3339_str_e());

    process_tick(&mut wtr_svc, time_tick, &String::from("process time tick"));
    let new_start = old_start.add_days(1);
    let cycle_start = &wtr_svc.engine.cycles[cycle_ptr as usize].schedule.start;
    println!("aplicação: {} - previsto: {}", &cycle_start.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e());
    // como o wizard torna dificil um oraculo sem reutilizar as mesmas funções, e como isto desde que esteja dentro dos 10' do nascer do sol, está bom
    // avançamos assim
    assert!((cycle_start.0 as i64 - new_start.0 as i64) < GIGA_I * 600, "nova data de arranque após recuperação diff < 10'");
    assert!(wtr_svc.get_mode() == Mode::Wizard, "mode after recover");
    assert!(wtr_svc.engine.wtr_cfg.state == State::WzrWait, "state after recover");
    time_tick = time_tick.add_secs(1800);
    //avançamos + 12 horas e não deve mudar o modo
    process_tick(&mut wtr_svc, time_tick, &String::from("advance time"));
    assert!(wtr_svc.get_mode() == Mode::Wizard, "mode while waiting");
    assert!(wtr_svc.engine.wtr_cfg.state == State::WzrWait, "state while waiting recover");
    //e agora vamos reexecutar o ciclo no periodo/dia seguinte
    time_tick = wtr_svc.engine.cycles[cycle_ptr as usize].schedule.start;

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, &Vec::<bool>::new(), cycle_ptr as usize, State::WzrWait, Mode::Wizard, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_with_error_with_recover_000_084 finished");
}

fn prepare_wzrd_secs(wtr_svc: &mut WtrSvc, db: &Persist, start_up: &StartupData) {
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(wtr_svc, &[true, true, true, true, true, true], CtrlTime(0));
    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclos standard");
    assert_start_params(db.clone(), Mode::Wizard, &*wtr_svc, start_up.clone(), State::WzrWait);
}

fn prepare_wizard(start_time: CtrlTime, prev_water: CtrlTime, mode: Mode) -> (Persist, StartupData, CtrlTime, SMsgBrkr, JoinHandle<()>, WtrSvc) {
    let (db, start_up) = prepare_common(start_time, mode, prev_water);

    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    let dev_svc = Arc::new(DevicesSvc::new(&db, msg_broker.clone()));
    let weather_cfg = arc_rw(WthrCfg::new(db.clone(), start_time));
    let mut wtr_svc = WtrSvc::new(weather_cfg.read().alrt_thresholds.clone(), msg_broker.clone(), db.clone(), start_time, dev_svc);

    prepare_wzrd_secs(&mut wtr_svc, &db, &start_up);

    (db, start_up, prev_water, msg_broker, handle_evt_mng, wtr_svc)
}

fn run_skip_enhanced_wizard(time_ref: CtrlTime, prev_water: CtrlTime, secs_to_skip: &Vec<bool>) {
    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_wizard(time_ref, prev_water, Mode::Wizard);

    //vamos colocar os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &secs_to_skip, prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS, "zero ciclo standards");
    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap() as usize;

    let new_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
    let oracle_start_date = CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 01, 22, 5, 53, 49));
    assert_eq!(&oracle_start_date.as_rfc3339_str_e(), &new_start.as_rfc3339_str_e(), "data de arranque");

    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);
    let mut time_tick = new_start;

    time_tick = process_sectors_enhanced(&mut wtr_svc, time_tick, secs_to_skip, cycle_ptr, State::WzrWait, Mode::Wizard, 0);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
}

// ver o arranque com um ciclo cujas condições são para correr
// e todos os setores são executados execepto 1
#[test]
fn wtr_start_wizard_schedule_run_skip_n_sectors_000_063() {
    log_info!("test wtr_start_wizard_schedule_run_skip_n_sectors_000_085 started");
    let time_ref = CtrlTime::from_utc_parts(2022, 01, 22, 0, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 05, 52, 0);

    let mut combinations: Combinations<Range<usize>>;

    for nr_secs in 0..MAX_SECTORS {
        combinations = (0..MAX_SECTORS).combinations(nr_secs);
        for v in combinations {
            let mut secs_to_skip = vec![true, true, true, true, true, true];
            for idx in v {
                secs_to_skip[idx] = false;
            }
            println!("{:?}", &secs_to_skip);
            run_skip_enhanced_wizard(time_ref, prev_water, &secs_to_skip);
            drop(secs_to_skip);
        }
    }
    log_info!("test wtr_start_wizard_schedule_run_skip_n_sectors_000_085 finished");
}

fn prepare_wizard_no_mangle(
    start_time: CtrlTime, prev_water: CtrlTime, mode: Mode,
) -> (Persist, StartupData, CtrlTime, SMsgBrkr, JoinHandle<()>, WtrSvc) {
    let (db, start_up) = prepare_common(start_time, mode, prev_water);

    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    let dev_svc = Arc::new(DevicesSvc::new(&db, msg_broker.clone()));
    let weather_cfg = arc_rw(WthrCfg::new(db.clone(), start_time));
    let wtr_svc = WtrSvc::new(weather_cfg.read().alrt_thresholds.clone(), msg_broker.clone(), db.clone(), start_time, dev_svc);

    (db, start_up, prev_water, msg_broker, handle_evt_mng, wtr_svc)
}

fn wtr_start_wizard_schedule_all_secs(start_time: CtrlTime, end_time: CtrlTime, step_secs: u64, prev_water: CtrlTime) {
    log_info!("test wtr_start_wizard_schedule_all_secs started");

    let (db, start_up, time_tick, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_wizard_no_mangle(start_time, prev_water, Mode::Wizard);
    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    //criamos a maquina um pouco antes das 21:50 do dia 22 para calcular o tempo do ciclo para o dia certo
    let mut time_tick = time_tick;

    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclo standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    time_tick = time_tick.add_secs(1);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap() as usize;
    let cycle = &wtr_svc.engine.cycles[cycle_ptr];

    let cycle_start = cycle.schedule.start;
    let mut total_cycles = 0;
    let mut tempo_total_em_rega = 0;
    let mut prev_start: CtrlTime = cycle_start;
    let mut prev_is_active = false;
    let mut actual_is_active: bool; // = false;
    let mut iter_counter: usize = 0;
    let mut next_start: CtrlTime;
    let mut d: DateTimeE;
    let mut stored_month = 1;

    while time_tick < end_time {
        time_tick = time_tick.add_secs(step_secs);
        wtr_svc.verify_things_to_do(time_tick);
        next_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
        if wtr_svc.engine.active_ptrs.cycle.is_some() {
            actual_is_active = true;
        } else {
            actual_is_active = false;
        }
        if actual_is_active != prev_is_active {
            if actual_is_active {
                assert!(time_tick == next_start, "left: {}, right: {}", time_tick.as_rfc3339_str_e(), next_start.as_rfc3339_str_e());
                prev_start = time_tick;
                total_cycles += 1;
            }
            if !actual_is_active {
                tempo_total_em_rega += time_tick.0 - prev_start.0;
            }
            prev_is_active = actual_is_active;
        }
        if actual_is_active {
            if wtr_svc.engine.active_ptrs.sec_id.is_some() {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrSector);
            } else {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrCycle);
            }
        } else {
            assert_engine_state(&wtr_svc, iter_counter, time_tick, next_start, State::WzrWait);
        }
        iter_counter += 1;
        d = time_tick.as_utc_date_time_e();
        if stored_month != d.month {
            println!("Estamos em {}/{}/{}", d.year, d.month, d.day);
            stored_month = d.month;
        }
    }

    println!("Começou en:{}", start_time.as_rfc3339_str_e());
    println!("Nr total de ciclos: {}", total_cycles);
    println!("Tempo total em rega: {}", elapsed_dyn(tempo_total_em_rega));
    println!("Terminou em: {}", time_tick.as_rfc3339_str_e());

    terminate_and_wait(wtr_svc, time_tick, msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_all_secs finished");
}

// #[test]//só para correr de vez em quando
fn wtr_run_wizard_all_six_months_covered_000_064() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 01, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2021, 12, 31, 5, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 06, 31, 0, 0, 0);

    wtr_start_wizard_schedule_all_secs(start_time, end_time, 1, prev_water);
}

#[test]
fn wtr_run_wizard_all_one_week_covered_000_065() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 01, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2021, 12, 31, 5, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 01, 8, 0, 0, 0);

    wtr_start_wizard_schedule_all_secs(start_time, end_time, 1, prev_water);
}

pub fn create_data_for_day(data_time: CtrlTime, db: &Persist, rain: f32) {
    // criar os dados base do dia em causa
    // temos que criar rain para os dias em causa
    for i in 0..24 {
        // registamos um valor de hora a hora, só para ver a coisa funcionar...porque o et é carregado á frente diretamente
        let time = data_time + i * CtrlTime::NR_NANOS_IN_A_HOUR;
        let values = Values { today_rain: rain, temp: 20., wind_b: 0., wind_i: 10., hr: 0.5, press: 1000. };
        ins_sensor_data(time, values, db);
    }
    // temos que criar et para o dia em causa
    let mut daily_metrics_buf = ArrayVec::<SensorValue, 1>::new();
    let time = data_time.sod_ux_e();

    daily_metrics_buf.push(SensorValue::new(Metric::EvapoTranspiration as u8, time, ORACLE_ET));

    _ = db.ins_daily_data_batch_aux(&daily_metrics_buf);
}

pub fn create_data_for_day_list(data_since: CtrlTime, db: &Persist, day_list: &[u64], rain: f32) {
    // criar os dados base do dia em causa
    // temos que criar rain para os dias em causa
    for d in day_list {
        create_data_for_day(data_since.add_days(*d), db, rain);
    }
}

fn wtr_start_wizard_schedule_skipping_secs(
    start_time: CtrlTime, end_time: CtrlTime, step_secs: u64, prev_water: CtrlTime, day_list: &[u64], rain: f32,
) {
    log_info!("test wtr_start_wizard_schedule_skipping_secs started");

    // estabelecer as condições iniciais dos setores para podermos validar as contas
    let db = Persist::new();

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(CtrlTime(0).ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    //estabelecer as condições dos setores
    for sec in sectors.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
        _ = db.upd_sec(&sec);
    }

    //agora criamos os objectos
    let (db, start_up, time_tick, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_wizard_no_mangle(start_time, prev_water, Mode::Wizard);

    // criação das condições
    wtr_svc.engine.wtr_cfg.live_since = start_time.sub_days(5);
    wtr_svc.engine.wtr_cfg.fresh_start = 1;
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();
    // criamos os dados
    // em tese rega no dia 0, salta o dia 1 porque á chuva, e rega nos outros dias..a lista começa em 2 porque subtraimos aqui 1 dia
    create_data_for_day_list(start_time.sub_days(1).sod_ux_e(), &db, day_list, rain); //em 24 horas dará 39 mm...esperemos :-)

    let mut time_tick = time_tick;
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclo standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    time_tick = time_tick.add_secs(1);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap() as usize;
    let cycle = &wtr_svc.engine.cycles[cycle_ptr];

    let cycle_start = cycle.schedule.start;
    let mut total_cycles = 0;
    let mut tempo_total_em_rega = 0;
    let mut prev_start: CtrlTime = cycle_start;
    let mut prev_is_active = false;
    let mut actual_is_active: bool; // = false;
    let mut iter_counter: usize = 0;
    let mut next_start: CtrlTime;
    let mut d: DateTimeE;
    let mut stored_month = 1;

    let mut displayed_sector = false;
    let mut displayed_cycle = false;

    while time_tick < end_time {
        time_tick = time_tick.add_secs(step_secs);
        wtr_svc.verify_things_to_do(time_tick);
        next_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
        if wtr_svc.engine.active_ptrs.cycle.is_some() {
            actual_is_active = true;
            if !displayed_cycle {
                println!("a regar o ciclo: {} iniciado em: {}", wtr_svc.engine.active_ptrs.cycle.unwrap(), time_tick.as_rfc3339_str_e());
                displayed_cycle = true;
                for sec in wtr_svc.engine.sectors.iter() {
                    println!("deficit do setor {}: {}", sec.id, sec.deficit)
                }
            }
        } else {
            actual_is_active = false;
            displayed_cycle = false;
        }
        if actual_is_active != prev_is_active {
            if actual_is_active {
                assert!(time_tick == next_start, "left: {}, right: {}", time_tick.as_rfc3339_str_e(), next_start.as_rfc3339_str_e());
                prev_start = time_tick;
                total_cycles += 1;
            }
            if !actual_is_active {
                tempo_total_em_rega += time_tick.0 - prev_start.0;
                for sec in wtr_svc.engine.sectors.iter() {
                    println!("deficit do setor {}: {}", sec.id, sec.deficit)
                }
            }
            prev_is_active = actual_is_active;
        }
        if actual_is_active {
            if wtr_svc.engine.active_ptrs.sec_id.is_some() {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrSector);
                if !displayed_sector {
                    println!("a regar o setor: {}", wtr_svc.engine.active_ptrs.sec_id.unwrap());
                    displayed_sector = true;
                }
            } else {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrCycle);
                displayed_sector = false;
            }
        } else {
            assert_engine_state(&wtr_svc, iter_counter, time_tick, next_start, State::WzrWait);
            displayed_sector = false;
        }
        iter_counter += 1;
        d = time_tick.as_utc_date_time_e();
        if stored_month != d.month {
            println!("Estamos em {}/{}/{}", d.year, d.month, d.day);
            stored_month = d.month;
        }
    }

    println!("Começou em:{}", start_time.as_rfc3339_str_e());
    println!("Nr total de ciclos: {}", total_cycles);
    println!("Tempo total em rega: {}", elapsed_dyn(tempo_total_em_rega));
    println!("Terminou em: {}", time_tick.as_rfc3339_str_e());

    terminate_and_wait(wtr_svc, time_tick, msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_skipping_secs finished");
}

// ver o arranque com um ciclo cujas condições são para correr
// e todos os setores são executados execepto n
// e estes n são executados no ciclo seguinte porque já há condições
#[test]
fn wtr_start_wizard_run_skip_6_sectors_with_cycle_creation_000_066() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 01, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2021, 12, 31, 5, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 01, 8, 0, 0, 0);

    wtr_start_wizard_schedule_skipping_secs(start_time, end_time, 1, prev_water, &[1, 4, 5, 6, 7], 1.);
}

fn wtr_start_wizard_schedule_with_alert(
    start_time: CtrlTime, end_time: CtrlTime, step_secs: u64, prev_water: CtrlTime, day_list: &[u64], rain: f32, alert: Alert,
) {
    log_info!("test wtr_start_wizard_schedule_skipping_secs started");

    // estabelecer as condições iniciais dos setores para podermos validar as contas
    let db = Persist::new();

    // estabelecemos as condições do arranque
    _ = db.update_param_gen(Module::Water as u8, WateringParams::LastSave as u8, (None, Some(CtrlTime(0).ux_ts()), None));
    _ = db.update_param_gen(Module::Water as u8, WateringParams::FreshStart as u8, (None, Some(0), None));

    let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
    db.get_cfg_secs(&mut sectors).unwrap();
    //estabelecer as condições dos setores
    for sec in sectors.iter_mut() {
        sec.deficit = 0.;
        sec.last_change = CtrlTime(0);
        sec.last_watered_in = CtrlTime(0);
        _ = db.upd_sec(&sec);
    }

    //agora criamos os objectos
    let (db, start_up, time_tick, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_wizard_no_mangle(start_time, prev_water, Mode::Wizard);

    // criação das condições
    wtr_svc.engine.wtr_cfg.live_since = start_time.sub_days(5);
    wtr_svc.engine.wtr_cfg.fresh_start = 1;
    // limpamos os dados dos outros testes
    _ = db.delete_daily_measures();
    _ = db.delete_metrics();
    // criamos os dados
    // em tese rega no dia 0, salta o dia 1 porque á chuva, e rega nos outros dias..a lista começa em 2 porque subtraimos aqui 1 dia
    create_data_for_day_list(start_time.sub_days(1).sod_ux_e(), &db, day_list, rain); //em 24 horas dará 39 mm...esperemos :-)

    let mut time_tick = time_tick;
    //vamos colocar todos os setores desejados disabled
    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);
    assert_eq!(wtr_svc.engine.cycles.len(), 2, "sem ciclo standard");
    assert_start_params(db.clone(), Mode::Wizard, &wtr_svc, start_up.clone(), State::WzrWait);

    time_tick = time_tick.add_secs(1);

    let cycle_ptr = wtr_svc.engine.internal.wizard.unwrap() as usize;
    let cycle = &wtr_svc.engine.cycles[cycle_ptr];

    let cycle_start = cycle.schedule.start;
    let mut total_cycles = 0;
    let mut tempo_total_em_rega = 0;
    let mut prev_start: CtrlTime = cycle_start;
    let mut prev_is_active = false;
    let mut actual_is_active: bool; // = false;
    let mut iter_counter: usize = 0;
    let mut next_start: CtrlTime;
    let mut d: DateTimeE;
    let mut stored_month = 1;

    let mut displayed_sector = false;
    let mut displayed_cycle = false;
    let mut in_alert = false;
    let mut prev_in_alert: bool = false;
    let mut nr_of_alerts = 0;
    let mut start_suspend = CtrlTime(0);
    let mut time_in_suspend: u64;
    let mut still_to_process_alert_request: bool;

    while time_tick < end_time {
        time_tick = time_tick.add_secs(step_secs);
        wtr_svc.verify_things_to_do(time_tick);
        still_to_process_alert_request = false;
        next_start = wtr_svc.engine.cycles[cycle_ptr].schedule.start;
        if wtr_svc.engine.active_ptrs.cycle.is_some() {
            actual_is_active = true;
            if !displayed_cycle {
                println!("a regar o ciclo: {} iniciado em: {}", wtr_svc.engine.active_ptrs.cycle.unwrap(), time_tick.as_rfc3339_str_e());
                displayed_cycle = true;
                for sec in wtr_svc.engine.sectors.iter() {
                    println!("deficit do setor {}: {}", sec.id, sec.deficit)
                }
            }
        } else {
            actual_is_active = false;
            displayed_cycle = false;
        }
        if actual_is_active != prev_is_active {
            if actual_is_active {
                assert!(time_tick == next_start, "left: {}, right: {}", time_tick.as_rfc3339_str_e(), next_start.as_rfc3339_str_e());
                prev_start = time_tick;
                total_cycles += 1;
            }
            if !actual_is_active {
                tempo_total_em_rega += time_tick.0 - prev_start.0;
                for sec in wtr_svc.engine.sectors.iter() {
                    println!("deficit do setor {}: {}", sec.id, sec.deficit)
                }
            }
            prev_is_active = actual_is_active;
        }
        if actual_is_active {
            if wtr_svc.engine.active_ptrs.sec_id.is_some() && wtr_svc.engine.suspend_timeout.is_none() {
                assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrSector);
                if !displayed_sector {
                    println!("a regar o setor: {}", wtr_svc.engine.active_ptrs.sec_id.unwrap());
                    displayed_sector = true;
                    //alerta mas só no primeiro ciclo
                    if wtr_svc.engine.active_ptrs.sec_id.unwrap() == 3 && !in_alert && !prev_in_alert {
                        // e agora enviar um alerta depois de arrancar o setor 3.  e só faz isso uma vez
                        start_suspend = time_tick;
                        wtr_svc.process_alert(alert.clone());
                        in_alert = true;
                        prev_in_alert = true;
                        nr_of_alerts += 1;
                        still_to_process_alert_request = true;
                    }
                }
            } else {
                if wtr_svc.engine.suspend_timeout.is_none() {
                    assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrCycle);
                    displayed_sector = false;
                    in_alert = false;
                } else {
                    assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::SuspendedWizard);
                }
            }
        } else {
            assert_engine_state(&wtr_svc, iter_counter, time_tick, next_start, State::WzrWait);
            displayed_sector = false;
        }
        if in_alert && wtr_svc.engine.suspend_timeout.is_some() {
            assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::SuspendedWizard);
        }
        if in_alert && wtr_svc.engine.suspend_timeout.is_none() && !still_to_process_alert_request && nr_of_alerts == 1 {
            // saiu de suspend por timeout e termina o ciclo
            assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWait);
            in_alert = false;
            prev_in_alert = false;
            time_in_suspend = (time_tick.0 - start_suspend.0) / CtrlTime::NR_NANOS_IN_A_MINUTE;
            println!("Entrou em suspend em: {}", start_suspend.as_rfc3339_str_e());
            println!("Saiu de suspend em: {}", time_tick.as_rfc3339_str_e());
            assert_eq!(wtr_svc.engine.wtr_cfg.wizard_info.suspend_timeout as u64, time_in_suspend, "número de minutos in suspend");
        }
        if in_alert && wtr_svc.engine.suspend_timeout.is_none() && !still_to_process_alert_request && nr_of_alerts == 2 {
            // saiu de suspend por resume e continua a rega
            assert_engine_state(&wtr_svc, iter_counter, time_tick, prev_start, State::WzrWtrSector);
            println!("a regar o setor: {}", wtr_svc.engine.active_ptrs.sec_id.unwrap());
            in_alert = false;
            prev_in_alert = false;
            time_in_suspend = (time_tick.0 - start_suspend.0) / CtrlTime::NR_NANOS_IN_A_MINUTE;
            println!("Entrou em suspend em: {}", start_suspend.as_rfc3339_str_e());
            println!("Saiu de suspend em: {}", time_tick.as_rfc3339_str_e());
            assert_eq!(20, time_in_suspend, "número de minutos in suspend");
        }
        if in_alert && wtr_svc.engine.suspend_timeout.is_some() && !still_to_process_alert_request && nr_of_alerts == 2 {
            if (time_tick.0 - start_suspend.0) / CtrlTime::NR_NANOS_IN_A_MINUTE >= 20 {
                //vamos fazer o resume
                let mut weather = Weather::default();
                weather.rain_period = 0.1;
                weather.wind_intensity = 10.;
                // nr_of_alerts += 1;
                wtr_svc.process_weather(&weather);
            }
        }
        iter_counter += 1;
        d = time_tick.as_utc_date_time_e();
        if stored_month != d.month {
            println!("Estamos em {}/{}/{}", d.year, d.month, d.day);
            stored_month = d.month;
        }
    }

    println!("Começou em:{}", start_time.as_rfc3339_str_e());
    println!("Nr total de ciclos: {}", total_cycles);
    println!("Tempo total em rega: {}", elapsed_dyn(tempo_total_em_rega));
    println!("Terminou em: {}", time_tick.as_rfc3339_str_e());

    terminate_and_wait(wtr_svc, time_tick, msg_broker, handle_evt_mng);
    log_info!("test wtr_start_wizard_schedule_skipping_secs finished");
}

// ver o arranque com um ciclo cujas condições são para correr
// o ciclo começa e a meio chega um alerta, colocando a maquina em suspend
// testamos isto para 2 ciclos
// no primeiro termina por timeout
// no segundo termina por instrução de resume antes do timeout
#[test]
fn wtr_start_wizard_run_suspend_with_wind_alert_000_067() {
    let start_time = CtrlTime::from_utc_parts(2022, 01, 01, 0, 30, 0);
    let prev_water = CtrlTime::from_utc_parts(2021, 12, 31, 5, 32, 0);
    let end_time = CtrlTime::from_utc_parts(2022, 01, 3, 0, 0, 0);

    wtr_start_wizard_schedule_with_alert(start_time, end_time, 1, prev_water, &[], 0., Alert::new(AlertType::RAIN, 0.5));
}

// -------  CHANGE MODE MANUAL PARA STANDARD OU WIZARD

// ver a mudança do modo manual para standard
#[test]
fn wtr_change_mode_manual_to_standard_not_working_000_068() {
    log_info!("test wtr_change_mode_manual_to_standard_not_working_000_068 started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    let mut time_tick = time_ref;

    // not working
    wtr_svc.verify_things_to_do(time_tick);

    // assert not working
    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::StdWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_change_mode_manual_to_standard_not_working_000_068 finished");
}

// ver a mudança do modo manual para standard - para o ciclo ativo e reconfigura
#[test]
fn wtr_change_mode_manual_to_standard_during_watering_one_sector_000_069() {
    log_info!("test wtr_change_mode_manual_to_standard_not_working_000_068 started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    let mut time_tick = time_ref;

    // not working
    wtr_svc.verify_things_to_do(time_tick);

    // assert not working
    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    //em tese isto arranca a rega
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ForceSector(0), State::ManWtrSectorDirect);

    // change mode
    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Standard), State::StdWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_change_mode_manual_to_standard_not_working_000_068 finished");
}

// ver a mudança do modo manual para wizard
#[test]
fn wtr_change_mode_manual_to_wizard_000_070() {
    log_info!("test wtr_change_mode_manual_to_wizard_000_070 started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, Mode::Manual);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    let mut time_tick = time_ref;

    // not working
    wtr_svc.verify_things_to_do(time_tick);

    // assert not working
    assert_start_params(db.clone(), Mode::Manual, &wtr_svc, start_up.clone(), State::ManWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Wizard), State::WzrWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_change_mode_manual_to_wizard_000_070 finished");
}

// -------  CHANGE MODE STANDARD PARA MANUAL OU WIZARD

// ver a mudança do modo standard para manual
#[test]
fn wtr_change_mode_standard_to_manual_not_working_000_071() {
    log_info!("test wtr_change_mode_standard_to_manual_not_working_000_071 started");

    let time_ref = CtrlTime::from_utc_parts(2022, 01, 23, 12, 0, 0);
    let prev_water = CtrlTime::from_utc_parts(2022, 01, 21, 21, 50, 0);

    let valid_start_mode = Mode::Standard;

    let (db, start_up, _prev_water, msg_broker, handle_evt_mng, mut wtr_svc) = prepare_standard(time_ref, prev_water, valid_start_mode);

    //estabelecer as condições dos setores
    for sec in wtr_svc.engine.sectors.iter_mut() {
        sec.last_watered_in = CtrlTime(0);
    }

    cfg_sectors_enabled(&mut wtr_svc, &[true, true, true, true, true, true], prev_water);

    assert_eq!(wtr_svc.engine.cycles.len(), MAX_INTERNALS + 1, "um ciclo standard existente");

    assert_start_params(db.clone(), valid_start_mode, &wtr_svc, start_up.clone(), State::StdWait);

    let mut time_tick = time_ref;

    // not working
    wtr_svc.verify_things_to_do(time_tick);

    // assert not working
    assert_start_params(db.clone(), valid_start_mode, &wtr_svc, start_up.clone(), State::StdWait);

    time_tick = send_and_apply_command(&mut wtr_svc, time_tick, Command::ChangeMode(Mode::Manual), State::ManWait);

    terminate_and_wait(wtr_svc, time_tick.add_secs(1), msg_broker, handle_evt_mng);
    log_info!("test wtr_change_mode_standard_to_manual_not_working_000_071 finished");
}

//>>>>>>>>>>>>>>>>>>>  VOU AQUI NOS TESTES  >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
// ver a mudança do modo standard para manual - para a rega e reconfigura
#[test]
fn wtr_change_mode_standard_to_manual_during_watering_000_072() {}

// ver a mudança do modo standard para wizard e pouco depois arranca um ciclo
#[test]
fn wtr_change_mode_standard_to_wizard_not_working_000_073() {}

// ver a mudança do modo manual para wizard durante um ciclo - para a rega e reconfigura
#[test]
fn wtr_change_mode_standard_to_wizard_during_watering_000_074() {}

// ver a mudança do modo standard para wizard com maquina em erro
// estando em erro, o modo wizard é ignorado
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_standard_to_wizard_in_error_000_075() {}

// ver a mudança do modo standard para standard com maquina em erro
// estando em erro, o modo wizard é ignorado
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_standard_to_standard_in_error_000_076() {}

// ver a mudança do modo standard para manual com maquina em erro
// estando em erro, esta é a unica alteração de estado/comando aceite
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_standard_to_manual_in_error_000_077() {}

// -------  CHANGE MODE WIZARD PARA MANUAL OU STANDARD

// ver a mudança do modo wizard para manual
#[test]
fn wtr_change_mode_wizard_to_manual_not_working_000_078() {}

// ver a mudança do modo wizard para manual - a maquina para a rega e reconfigura
#[test]
fn wtr_change_mode_wizard_to_manual_during_watering_000_079() {}

// ver a mudança do modo wizard para standard e pouco depois arranca um ciclo
#[test]
fn wtr_change_mode_wizard_to_standard_not_working_000_080() {}

// ver a mudança do modo wizard para standard edurante um ciclo - a maquina para a rega e reconfigura
#[test]
fn wtr_change_mode_wizard_to_standard_during_watering_000_081() {}

// ver a mudança do modo wizard para wizard com maquina em erro
// estando em erro, o modo wizard é ignorado
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_wizard_to_wizard_in_error_000_082() {}

// ver a mudança do modo wizard para standard com maquina em erro
// estando em erro, o modo standard é ignorado
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_wizard_to_standard_in_error_000_083() {}

// ver a mudança do modo wizard para manual com maquina em erro
// estando em erro, esta é a unica alteração de estado/comando aceite
// é uma "segurança" e uma forma de haver uma unica mudança de estado onde se faz a tentativa de resolução do erro para simplificar a coisa
#[test]
fn wtr_change_mode_wizard_to_manual_in_error_000_084() {}

// -------ERRORS
// maquina passa ao estado ERROR, tenta fechar da forma possivel a valvula, e envia error alert
#[test]
fn wtr_valve_error_during_close_000_085() {}

// maquina passa ao estado ERROR, tenta fechar da forma possivel, o que em principio nem abriu e  envia warning alert
#[test]
fn wtr_valve_error_during_open_000_086() {}

//quando muda o dia não é suposto acontecer nada na maquina de rega (a não ser esperar que o evento de mnt de bd termine, caso esteja a regar)
// se estiver a regar pode criar atraso a fechar ou abrir valvulas?  testar.
// se não estiver a regar pode criar atraso no inicio da rega - testar
#[test]
fn wtr_change_day_000_087() {}

#[test]
fn wtr_terminate_in_starting_000_088() {}

#[test]
fn wtr_wtr_terminate_in_no_schedule_def_000_089() {}

// este não sei se acontece porque é establish mode é um estado de passagem
#[test]
fn wtr_terminate_in_establish_mode_000_090() {}

#[test]
fn wtr_terminate_in_manual_wait_000_091() {}

#[test]
fn wtr_terminate_in_wizard_wait_000_092() {}

#[test]
fn wtr_terminate_in_standard_wait_000_093() {}

#[test]

fn wtr_terminate_in_manual_watering_cycle_000_094() {}

#[test]
fn wtr_terminate_in_standard_watering_cycle_000_095() {}

#[test]
fn wtr_terminate_in_wizard_watering_cycle_000_096() {}

#[test]
fn wtr_terminate_in_manual_watering_sector_000_097() {}

#[test]
fn wtr_terminate_in_standard_watering_sector_000_098() {}

#[test]
fn wtr_terminate_in_wizard_watering_sector_000_099() {}

#[test]
fn wtr_terminate_in_manual_watering_sector_direct_000_100() {}

#[test]
fn wtr_terminate_in_suspended_wizard_000_101() {}

#[test]
fn wtr_terminate_in_error_000_102() {}

#[test]
fn wtr_terminate_in_shutdown_000_103() {}

//-----------   LOAD TEST IN MAXIMUM WORKING CONDITIONS
// A IDEIA É TESTAR E AVALIAR O COMPORTAMENTO DA COISA EM CONDIÇÕES DE CARGA MÁXIMA
// CARGA MÁXIMA É A BASE DE DADOS CARREGADA COM O NR MAXIMO DE DIAS ANTES DA LIMPEZA A CADA X
//  X É CONFIGURAVEL, MAS ESTÁ PARA 20 DIAS, E ESTE ESTUDO VAI PERMITIR AFERIR O IMPACTO NA PERFORMANCE DA CARGA E SE NECESSÁRIO TER DADOS OBJETIVOS PARA AJUSTAR O X
// NO MODO WIZARD REGA A CADA 2 DIAS, TIPICAMENTE, MAS PODERÁ REGAR A CADA DIA (SE ESTIVER CALOR COMO Ó CARAÇAS)
// AS SITUAÇÕES DE SUSPEND TERÃO QUE SER TAMBÉM AVALIADAS- MAS ESSAS TIPICAMENTE REDUZEM O NR DE VEZES DA REGA, PELO QUE REDUZEM A CARGA
// PORTANTO TEREMOS 6 SETORES NA SECTORS
// 2 CICLOS NA SCHEDULED_CYCLES
// WATERED CYCLES SERÃO NO MÁXIMO 20, TIPICAMENTE, PODENDO SER UM POUCO MAIS SE REGAR MAIS DO QUE UMA VEZ POR DIA
// - ISTO LEVA PARA O TEMA DE QUE SE LIMITAMOS O HORARIO DA REGA WIZARD Á NOITE, O QUE FAZEMOS NOS GOLPES DE CALOR?
// REGAR AUTOMATICAMENTE VAI MOLHAR ALGUEM PORQUE PODE SER DURANTEO DIA E DE FORMA IMPREVISIVEL
// PORTANTO PARECE QUE O MELHOR É QUE NOS GOLPES DE CALOR
//, ENVIAR UM ALERTA PARA O CLIENTE A PERGUNTAR SE PODE REGAR ANTES DE REGAR....REVIEW: TEMA A IMPLEMENTAR DEPOIS DOS TESTES
// WATARED SECTORS SERÁ 6 SECTORS * 20 CYCLES NOS 20 DIAS PARAMETRIZADOS Á DATA, PORTANTO 120 REGISTOS
// A PERFORMANCE DISTO CRUZA COM O POTENCIAL IMPACTO QUE O MODULO WEATHER PODE TER,
// PORQUE ACEDE Á MESMA BD O QUE PODE POTENCIALMENTE CRIAR CONTENÇÃO SE A TABELA WEATHER ESTIVER MAIS PESADA....
// SE SEPARARMOS A BD WEATHER DA REGA, PODEMOS TER O TEMA DA CARGA - PERFORMANCE, MAS RESOLVE-SE O TEMA DA CONTENÇÃO NO ACESSO
// NESSE CASO A CONTENÇÃO POTENCIAL SERIA APENAS A TABELA DE PARAMETROS, MAS ESSA TAMBÉM PODE SER PARTIDA POR MODULO
// MAS O MODO WIZARD PRECISA DE IR BUSCAR A CHUVA Á TABELA DA METEROLOGIA, PELO QUE PARECE NÃO PARA PARA ESTA SEPARAÇÃO
// A TABELA WEATHER SÃO 9 SENSORES A MEDIR A CADA X SEGUNDOS - PARA A REGA O PERIODO PODE SER NA ORDEM DOS MINUTOS 6 POR EXEMPLO.
// PARA OUTRO TIPO DE SENSORES ISTO PODE SER SUPERIOR - TEMOS QUE TESTAR A CARGA PARA PERCEBER OS EFEITOS REAIS
// NO CENÁRIO REGA, A RECOLHER INFO PARA OS SENSORES A CADA 6 MINUTOS DÁ 9 SENSORES * 10 POR HORA * 24 HORAS * 20 DIAS = 43200 REGISTOS A CADA 20 DIAS
// PORTNTO TEM QUE SE TESTAR O GETRAIN COM CARGA DE 43200 REGISTOS E PARA PERCEBER O IMPACTO, JUSTAR ISTO PARA
// MAS TEMOS QUE TESTAR O COMPORTAMENTO DO SQLLITE NO GETRAIN COM OS SENSORES A LER EM DIFERENTES PERIODICIDADES, COM AS SEGUINTES REFERENCIAS
// 10 VEZES POR HORA        =>     43 200 LINHAS    => LOAD_CASE_1
// 1 VEZ POR MINUTO         =>    259 200 LINHAS    => LOAD_CASE_2
// 1 VEZ POR SEGUNDO        => 15 552 000 LINHAS    => LOAD_CASE_3
//
// MEDIR TAMBÉM OS TEMPOS DA MANUTENÇÃO NESTAS DIFERENTES CARGAS

use alloc_counter::count_alloc;

use super::db_irrigation::ins_sensor_data;

#[test]
fn test_take() {
    let y = Some(9);
    let z = Some(9);
    fn sem_buffer(o: Option<i32>) {
        let t0 = Instant::now();

        for _i in 0..100 {
            let _x = o;
        }
        println!("{:?}", o);
        println!("Sem buffer {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    }
    fn com_buffer(o: Option<i32>) {
        let t0 = Instant::now();

        let mut _h: Option<i32> = None;
        for _i in 0..100 {
            _h = std::mem::replace(&mut Some(10), o);
        }
        println!("{:?}", Some(10));
        println!("Com buffer {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    }
    let counts = count_alloc(|| sem_buffer(y));
    println!("Sem buffer        Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    let counts = count_alloc(|| com_buffer(z));
    println!("Com buffer        Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);
}

// testar passagem da int msg para o broker com copia de todos os dados ou só com passagem de uma box
#[test]
fn test_intmessage_box() {}
