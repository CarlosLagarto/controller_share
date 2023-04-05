#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]

use alloc_counter::count_alloc;

use core::time;

use std::sync::Arc;
use std::time::Instant;
use std::{
    thread::{self},
    time::Duration,
};

use ctrl_lib::data_structs::msgs::int_message::*;
use ctrl_lib::data_structs::concurrent_queue::*;
use ctrl_lib::services::msg_broker::{subscriber::*, msg_brkr_svc::*};
use ctrl_lib::{app_time::ctrl_time::*, utils::elapsed_dyn};

use crate::integration::common::*;

#[test]
fn broker_service_start_stop() {
    setup_start_time(CtrlTime::sys_time());
    let MSG_BROKER = MsgBrkr::new();

    initialize_evt_mng();

    let handle_evt_mng = MSG_BROKER.start();
    // let counts = count_alloc(|| MSG_BROKER.start());
    // println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    // assert!(MSG_BROKER.working(), "working flag devia ser true e é = {}", MSG_BROKER.working());
    thread::sleep(Duration::from_secs(1));
    // assert!(MSG_BROKER.working(), "1. event broker is running. started flag = {}", MSG_BROKER.working());

    let _res = MSG_BROKER.terminate();
    thread::sleep(Duration::from_secs(1));
    // assert!(!MSG_BROKER.working(), "2. event broker is stopped.  working flag devia ser false e é = {}", MSG_BROKER.working());
    let _res = handle_evt_mng.join();
    // assert!(true, "10. event broker thread finish");
}

#[test]
fn broker_service_start_stop_alloc_count() {
    setup_start_time(CtrlTime::sys_time());
    let MSG_BROKER = MsgBrkr::new();

    initialize_evt_mng();

    let t0 = Instant::now();
    let counts = count_alloc(|| MSG_BROKER.start());
    println!("tempo: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    let _res = MSG_BROKER.terminate();
}

#[test]
fn broker_service_process_alloc_count_no_subscribers() {
    setup_start_time(CtrlTime::sys_time());
    let MSG_BROKER = MsgBrkr::new();

    initialize_evt_mng();

    MSG_BROKER.start();
    let t0 = Instant::now();
    let counts =
        count_alloc(|| process_msg(Box::new(IntMessage::build(MsgData::CycleAdded, CtrlTime::sys_time())), &MSG_BROKER.inner));
    println!("tempo: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    let _res = MSG_BROKER.terminate();
}

#[test]
fn broker_service_process_alloc_count_one_subscribers() {
    setup_start_time(CtrlTime::sys_time());
    let MSG_BROKER = MsgBrkr::new();

    initialize_evt_mng();

    MSG_BROKER.start();
    let _broker_channel = MSG_BROKER.subscribe(MsgType::ShutDown, Subscriber::Test);
    let _broker_channel = MSG_BROKER.subscribe(MsgType::CycleAdded, Subscriber::Test);
    let msg_data = MsgData::ShutDown(CtrlTime::sys_time());
    let msg = IntMessage::build(msg_data, CtrlTime::sys_time());
    let t0 = Instant::now();
    let counts = count_alloc(|| process_msg(Box::new(msg), &MSG_BROKER.inner));
    println!("tempo: {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    let _res = MSG_BROKER.terminate();
}

#[test]
fn broker_service_new_event() {
    setup_start_time(CtrlTime::sys_time());
    let MSG_BROKER = MsgBrkr::new();
    initialize_evt_mng();

    let handle_evt_mng = MSG_BROKER.start();
    // assert!(MSG_BROKER.working(), "working flag devia ser true e é = {}", MSG_BROKER.working());

    let _res = MSG_BROKER.terminate();
    thread::sleep(Duration::from_secs(1));
    // assert!(!MSG_BROKER.working(), "2. event broker is stopped.  working flag devia ser false e é = {}", MSG_BROKER.working());
    let _res = handle_evt_mng.join();
    // assert!(true, "10. event broker thread finish");
}

#[test]
fn service_start_stop_via_ctrlc() {
    setup_start_time(CtrlTime::sys_time());

    let MSG_BROKER = MsgBrkr::new();
    initialize_evt_mng();

    let handle_evt_mng = MSG_BROKER.start();
    let recv_queue = Arc::new(MtDeque::new());
    _ = MSG_BROKER.register_in_broker(Subscriber::Test, recv_queue.clone());
    _ = MSG_BROKER.subscribe(MsgType::ShutDown, Subscriber::Test);
    let _thread1 = std::thread::spawn(move || {
        let _msg = recv_queue.recv();
        println!("5. shutdown received do ctrl-c");
    });
    let broker_clone = Arc::new(MSG_BROKER);
    let ref_broker = broker_clone.clone();
    let _thread2 = std::thread::spawn(move || {
        thread::sleep(time::Duration::from_secs_f32(2.));
        let _res = broker_clone.reg_int_msg(MsgData::ShutDown(CtrlTime::sys_time()), CtrlTime::sys_time());
        println!("4. shutdown issued do ctrl-c");
    });

    _thread1.join();
    _thread2.join();

    let _res = ref_broker.terminate();
    let _res = handle_evt_mng.join();

    println!("conclui com sucesso");
}
