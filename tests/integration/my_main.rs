use std::{thread, time::Duration, sync::Arc};

use controller::{signal_handler::install_signal_handler, run::*};
use ctrl_lib::{config::log_config::AppLog, utils::TESTING, logger::initialize_logger};

use crate::integration::common::*;

use std::sync::atomic::AtomicBool;

#[test]
fn test_signal_handler() {
    let term_now = Arc::new(AtomicBool::new(false));
    let (_sigs_info, sigs) = install_signal_handler(term_now).unwrap();
    // assert!(_exit_code == 0);
    assert!(sigs.len() == 4, "o comprimento é: {}", sigs.len());
}

#[test]
#[ignore]
fn test_run_program(){
    
    // isto é só para carregar a configuração para sabermos se estamos em teste ou não, e configurar os ports de forma fdecente
    let app_log = AppLog::new();
    let _logger_handle = initialize_logger(&app_log);
    if app_log.test_in_progress == 1 {
        unsafe { TESTING = true };
    }

    //tenho que lançar numa thread diferente
    let _thread_program = std::thread::spawn(move || {
        run();
    });
    //e depois faço aqui sleep por algum tempo - ou interajo com o programa
    thread::sleep(Duration::from_secs(3));

    // e os comandos serão por web para comandar o programa.
    // nice!   começou por ser um canal alternativo de controlo do programa, e revelou-se importante para testar a coisa.  // nice!!!!
    let _res = make_request("http://localhost:5004/tst/controller/shutdown");

    let _res = _thread_program.join();
}

#[test]
#[ignore]
fn test_run_program_to_catch_weather_stack_overflow(){
    
    //tenho que lançar numa thread diferente
    let _thread_program = std::thread::spawn(move || {
        run();
    });
    //e depois faço aqui sleep por algum tempo - ou interajo com o programa
    thread::sleep(Duration::from_secs(20));

    // e os comandos serão por web para comandar o programa.
    // nice!   começou por ser um canal alternativo de controlo do programa, e revelou-se importante para testar a coisa.  // nice!!!!
    let _res = make_request("http://localhost:5004/tst/controller/shutdown");

    let _res = _thread_program.join();
}