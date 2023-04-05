use ctrl_lib::logger::*;
use ctrl_lib::services::{db_maint::db_mnt_svc::*, msg_broker::msg_brkr_svc::*};
use ctrl_lib::{app_time::ctrl_time::*, config::app_config::*, db::db_sql_lite::*, log_info};
use ctrl_prelude::globals::SHUTTING_DOWN;

// test execução pela primeira vez
// inicializa o daily e o periodico
// corre o daily no fim do dia
#[test]
fn db_mnt_first_time_run() {
    log_info!("test db_mnt_first_time_run started");

    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db.clone());
    let msg_broker = MsgBrkr::new();
    let handle_evt_mng = msg_broker.start();

    let mut time = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    app_cfg.db_maint.db_mnt_last_run = CtrlTime(0);
    app_cfg.db_maint.db_mnt_counter = 0;
    let mut db_mnt = DBMntSvc::new(time, &mut app_cfg);

    assert_eq!(app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(), CtrlTime(0).as_rfc3339_str_e(), "first run should be  0");
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "next daily at end of day"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - 1;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("antes de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);

    //E correndo na mesma hora, nada acontece porque ainda não é a hora de nenhum dos schedules
    assert_eq!(app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(), CtrlTime(0).as_rfc3339_str_e(), "Ainda não correu pelo que last run deve ser 0");
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "A proxima execução do daily é no fim do dia"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - 1;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("depois de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    time = adjust_time_for_delay(time.eod_ux_e()); //avançamos para o fim do dia que é quando deve começar a correr....
    println!("depois de correr o daily - a proxima execução do large é a {}", db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e());
    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);
    println!("depois de correr o daily - a proxima execução do large é a {}", db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e());

    //E agora deve ter corrido o daily
    assert_eq!(app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(), time.as_rfc3339_str_e(), "deve ser igual ao time de agora");
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        time.add_days(1).as_rfc3339_str_e(),
        "A proxima execução do daily é no dia seguinte"
    );
    assert_eq!(app_cfg.db_maint.db_mnt_counter, 1, "deve ter incrementado o counter");

    println!("a proxima execução do large é esperemos a mesma {}", db_mnt.large_mnt_schedule.start.as_rfc3339_str_e());
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é esperemos a mesma {}",
        next_large_exec.as_rfc3339_str_e()
    );

    unsafe {SHUTTING_DOWN = true};
    msg_broker.terminate();
    let _res = handle_evt_mng.join();

    log_info!("test db_mnt_first_time_run finished");
}

// test execução pela primeira vez depois de paragem
// cenario onde parou pela ultima vez quando ainda faltam dias para o periodico
#[test]
fn db_mnt_interrupted_time_run_not_periodic_day() {
    log_info!("test db_mnt_interrupted_time_run_not_periodic_day started");

    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db.clone());
    let msg_broker = MsgBrkr::new();
    let handle_evt_mng = msg_broker.start();

    let nr_of_days_since_last_run = 5;
    let mut time = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    app_cfg.db_maint.db_mnt_last_run = adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)); //correu pela ultima vez á 4 dias (agora será o 5º)
    app_cfg.db_maint.db_mnt_counter = nr_of_days_since_last_run as u8; // é indiferente aqui o nr, desde que seja < 19, para este teste
    let mut db_mnt = DBMntSvc::new(time, &mut app_cfg);

    assert_eq!(
        app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)).as_rfc3339_str_e(),
        "interrupted run should keep last value"
    );
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "next daily at end of day"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - app_cfg.db_maint.db_mnt_counter as u64;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("antes de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);

    //E correndo na mesma hora, nada acontece porque ainda não é a hora de nenhum dos schedules
    assert_eq!(
        app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)).as_rfc3339_str_e(),
        "Ainda não correu pelo que last run deve ser igual ao anterior"
    );
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "A proxima execução do daily é no fim do dia"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - app_cfg.db_maint.db_mnt_counter as u64;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("depois de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    time = adjust_time_for_delay(time.eod_ux_e()); //avanamos para o fim do dia que é quando deve começar a correr....

    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);

    //E agora deve ter corrido o daily
    assert_eq!(app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(), time.as_rfc3339_str_e(), "deve ser igual ao time de agora");
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        time.add_days(1).as_rfc3339_str_e(),
        "A proxima execução do daily é no dia seguinte"
    );
    assert_eq!(app_cfg.db_maint.db_mnt_counter, nr_of_days_since_last_run as u8 + 1, "deve ter incrementado o counter");

    println!("a proxima execução do large é esperemos a mesma {}", db_mnt.large_mnt_schedule.start.as_rfc3339_str_e());
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é esperemos a mesma {}",
        next_large_exec.as_rfc3339_str_e()
    );
    unsafe {SHUTTING_DOWN = true};
    msg_broker.terminate();
    let _res = handle_evt_mng.join();

    log_info!("test db_mnt_interrupted_time_run_not_periodic_day finished");
}

// test execução pela primeira vez depois de paragem
// cenario onde parou pela ultima vez no dia em que era para executar o periodico
#[test]
fn db_mnt_interrupted_time_run_periodic_day() {
    log_info!("test db_mnt_interrupted_time_run_not_periodic_day started");

    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db.clone());
    let msg_broker = MsgBrkr::new();
    let handle_evt_mng = msg_broker.start();

    let nr_of_days_since_last_run = 20;
    let mut time = CtrlTime::from_utc_parts(2022, 01, 01, 21, 50, 0);
    app_cfg.db_maint.db_mnt_last_run = adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)); //correu pela ultima vez á 19 dias (agora será o 5º)
    app_cfg.db_maint.db_mnt_counter = nr_of_days_since_last_run as u8; //
    let mut db_mnt = DBMntSvc::new(time, &mut app_cfg);

    assert_eq!(
        app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)).as_rfc3339_str_e(),
        "interrupted run should keep last value"
    );
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "next daily at end of day"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - app_cfg.db_maint.db_mnt_counter as u64;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("antes de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);

    //E correndo na mesma hora, nada acontece porque ainda não é a hora de nenhum dos schedules
    assert_eq!(
        app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e().sub_days(nr_of_days_since_last_run)).as_rfc3339_str_e(),
        "Ainda não correu pelo que last run deve ser igual ao anterior"
    );
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        adjust_time_for_delay(time.eod_ux_e()).as_rfc3339_str_e(),
        "A proxima execução do daily é no fim do dia"
    );

    let nr_of_days = app_cfg.db_maint.db_mnt_days as u64 - app_cfg.db_maint.db_mnt_counter as u64;
    let next_large_exec = adjust_time_for_delay(time.eod_ux_e()).add_days(nr_of_days);
    println!("depois de correr o daily - a proxima execução do large é daqui a {}", nr_of_days);
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.as_rfc3339_str_e(),
        "a proxima execução do large é daqui a {}",
        nr_of_days
    );

    time = time.add_days(1);
    time = adjust_time_for_delay(time.eod_ux_e()); //avançamos para o fim do dia que é quando deve começar a correr....

    db_mnt.verify_things_to_do(time, &msg_broker, &mut app_cfg);

    //E agora deve ter corrido o daily
    assert_eq!(app_cfg.db_maint.db_mnt_last_run.as_rfc3339_str_e(), time.as_rfc3339_str_e(), "deve ser igual ao time de agora");
    assert_eq!(
        db_mnt.daily_mnt_schedule.start.as_rfc3339_str_e(),
        time.add_days(1).as_rfc3339_str_e(),
        "A proxima execução do daily é no dia seguinte"
    );
    assert_eq!(app_cfg.db_maint.db_mnt_counter, 0, "deve ter reseted do counter");

    println!("a proxima execução do large é esperemos a mesma {}", db_mnt.large_mnt_schedule.start.as_rfc3339_str_e());
    assert_eq!(
        db_mnt.large_mnt_schedule.start.as_rfc3339_str_e(),
        next_large_exec.add_days(app_cfg.db_maint.db_mnt_days as u64).as_rfc3339_str_e(),
        "a proxima execução do large é esperemos a mesma {}",
        next_large_exec.as_rfc3339_str_e()
    );
    unsafe {SHUTTING_DOWN = true};
    msg_broker.terminate();
    let _res = handle_evt_mng.join();

    log_info!("test db_mnt_interrupted_time_run_not_periodic_day finished");
}
