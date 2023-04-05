#![allow(non_snake_case)]
use std::{
    sync::{Arc, atomic::Ordering},
    thread,
    time::{self, Duration},
};

use ctrl_lib::services::{mqtt::{mqtt_svc::*, paho::IS_CONNECTED}, msg_broker::msg_brkr_svc::*};
use ctrl_lib::{app_context::start_up::*, utils::*};

use crate::integration::common::*;
use ctrl_prelude::globals::SHUTTING_DOWN;

// OK, temos aqui o tema de que o MQTT é estático por causa do acesso á biblioteca em C do mqtt.
// O tema é que quando uma das trheads dealoca o objeto, as outras ficam em estado inconssistente.
// Em produção não é tema porque todas as threads chamam o mesmo objeto (by design)
// Como a infraestrutura de testes do Cargo Test lança os testes em paralelo, temos que fazer aqui ajustes para serializar a execução destes testes

#[allow(clippy::assertions_on_constants)]
#[test]
fn validate_mqtt_service_start_stop() {
    println!("");
    let mut t0 = time::Instant::now();
    let MSG_BROKER = Arc::new(MsgBrkr::new());
    println!("create broker: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    initialize_evt_mng();
    let _startup_data = StartupData::default();
    t0 = time::Instant::now();
    let broker_handle = MSG_BROKER.start();
    println!("start broker: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));

    thread::sleep(time::Duration::from_secs(1));
    // assert!(MSG_BROKER.working(), "1. event broker is running - working deve ser true e é {}", MSG_BROKER.working());
    t0 = time::Instant::now();
    let mut mqtt_svc: MQTTService = MQTTService::new(MSG_BROKER.clone());
    println!("create mqtt: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    t0 = time::Instant::now();
    let mqtt_handler = mqtt_svc.start();
    println!("start mqtt: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    let mut counter = 0;
    while !IS_CONNECTED.load(Ordering::Relaxed)  {
        thread::sleep(Duration::from_millis(5));
        counter += 1;
        if counter == 5000{
            println!("o mqtt não conseguiu ligar-se");
            break;
        }
    }
    unsafe {SHUTTING_DOWN = true };
    t0 = time::Instant::now();
    mqtt_svc.terminate();
    println!("mqtt service terminate: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));

    let res = mqtt_handler.join();
    if res.is_err() {
        assert!(false, "X1. mqtt service panic");
    } 

    // else {
    //     //depois de emitir o comando stop, é suposto a thread do web server já ter terminado,
    //     //pelo que esta - handler_web -  é também suposto já ter terminado
    //     //como o web server lê pedidos a cada segundo, 1 segundo + 50 milisegundos deve chegar...
    //     assert!(t0.elapsed() < Duration::from_millis(2050), "mqtt service stoped!: {:?}", t0.elapsed());
    //     //pelo que medi, leva entre 10 a 17 mili secs
    // }

    t0 = time::Instant::now();
    let _res = MSG_BROKER.terminate();
    let res = broker_handle.join();
    if res.is_err() {
        assert!(false, "X1. mqtt service panic");
    } 
    println!("broker service terminate: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    // if counter == 5000{
    //     assert!(false,"não conseguiu ligar ao mqtt");
    // }
}
