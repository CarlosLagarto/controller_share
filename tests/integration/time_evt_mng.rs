#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ctrl_lib::{
    app_time::ctrl_time::*,
    controller_sync::{DB_MAINT_SIG, NEW_DAY_SIG},
    data_structs::{concurrent_queue::MtDeque, msgs::int_message::*},
    services::msg_broker::{subscriber::*, msg_brkr_svc::*, *},
    utils::*,
};
use ctrl_prelude::globals::*;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{self, Duration},
};

use crate::integration::common::*;

fn validate_stop(handle_evt_mng: JoinHandle<()>, msg_broker: &MsgBrkr) {
    let _res = msg_broker.terminate();
    thread::sleep(Duration::from_secs(1));
    // assert!(!msg_broker.working(), "2. event broker is stopped.  working flag devia ser false e é = {}", msg_broker.working());
    let _res = handle_evt_mng.join();
    println!("10. event broker thread finish");
}

#[test]
fn service_iteraction_broker_timer() {
    // inicializa time
    setup_start_time(CtrlTime::from_utc_parts(2022, 1, 14, 22, 59, 55));
    let MSG_BROKER = MsgBrkr::new();

    //inicializa broker
    initialize_evt_mng();
    let handle_evt_mng = MSG_BROKER.start();
    let subs_queue = Arc::new(MtDeque::<IntMessage>::new());
    let _broker_channel = MSG_BROKER.register_in_broker(Subscriber::Test, subs_queue.clone());
    MSG_BROKER.subscribe(MsgType::ShutDown, Subscriber::Test);

    //vai ver...
    thread::sleep(time::Duration::from_secs_f32(0.5));
    println!("1. event broker is running");

    // thread que recebe eventos do time e do broker
    let broker_rx2 = subs_queue.clone();
    let _thread1 = std::thread::spawn(move || {
        // let mut rs: Vec<EventReceiver> = Vec::with_capacity(1);
        // rs.push(broker_rx2);

        loop {
            let result = broker_rx2.recv();
            if let Some(msg) = result {
                match msg.data {
                    // MsgData::TimeSignal(_) => (),
                    // MsgData::ShutDown(_) => {
                    //     break; // exit main listener, end thread , free mutex, and the join waiting in main thread awakes.
                    // }
                    MsgData::StateChanged => {
                        // para testar outros eventos no broker ao mesmo tempo que se recebe do time
                    }
                    _ => {
                        println!("Unforeseen event!");
                        break;
                    }
                }
            } else {
                break;
            }
        }
    });

    let n = 10;
    //thread que envia eventos para o broker
    let local_broker = Arc::new(MSG_BROKER);
    let ref_broker = local_broker.clone();
    let _thread2 = std::thread::spawn(move || {
        let n1 = n;
        for i in 1..n1 + 1 {
            thread::sleep(time::Duration::from_secs_f32(0.5));
            let _res = local_broker.reg_int_msg(MsgData::StateChanged, CtrlTime::sys_time());
            println!("enviou evento restart {} de {}", i, n);
        }
    });

    //aguarda ciclo
    let _res = _thread2.join();

    //envia comando para shutdown
    let wait_secs: f32 = (n + 1) as f32 * 0.5;
    thread::sleep(time::Duration::from_secs_f32(wait_secs));
    let _res = ref_broker.reg_int_msg(MsgData::ShutDown(CtrlTime::sys_time()), CtrlTime::sys_time());
    println!("enviou msg para terminar thread main");

    // espera que main thread termine
    let _res = _thread1.join();

    //para broker
    let _res = ref_broker.terminate();
    let _res = handle_evt_mng.join();

    println!("conclui teste com sucesso");
}
