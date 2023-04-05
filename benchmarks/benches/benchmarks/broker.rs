#![allow(dead_code)]
use criterion::{black_box, Criterion};//BenchmarkId, 
use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::data_structs::concurrent_queue::MtDeque;
use ctrl_lib::data_structs::concurrent_queue::SMtDeque;

use std::hash::*;
use std::sync::Arc;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
use std::thread;
use std::time::Duration;

use nohash_hasher::IntMap;

// use parking_lot::RwLock;
// pub use no_deadlocks::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use ctrl_lib::data_structs::msgs::int_message::*;
use ctrl_lib::{log_info, logger::*};
use ctrl_prelude::string_resources::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

#[repr(u8)]
#[derive(Eq, PartialEq, Hash)]
pub enum Subscriber {
    Main = 0,
    Mqtt = 1,
    Test = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Subscriber2 {
    Main = 0,
    Mqtt = 1,
    Test = 2,
}
pub const MAX_SUBSCRIBERS: usize = 3; //Não tenho á data (13/Jun/2022) mais do que 2 subscribers por tipo msg.  Qd tiver, é alterar aqui.

impl Subscriber {
    pub fn to_string<'a>(&self) -> &'a str {
        match *self {
            Subscriber::Main => "Main",
            Subscriber::Mqtt => "MQTT",
            Subscriber::Test => "Test",
        }
    }
}

impl Subscriber2 {
    pub fn to_string<'a>(&self) -> &'a str {
        match *self {
            Subscriber2::Main => "Main",
            Subscriber2::Mqtt => "MQTT",
            Subscriber2::Test => "Test",
        }
    }
}

impl std::hash::Hash for Subscriber2 {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u8(*self as u8)
    }
}

impl nohash_hasher::IsEnabled for Subscriber2 {}

type MsgSubscribedList = SmallVec<[Subscriber; MAX_SUBSCRIBERS]>;
type MsgSubscribedList1 = SmallVec<[Subscriber1; MAX_SUBSCRIBERS]>;
type MsgSubscribedList2 = SmallVec<[Subscriber2; MAX_SUBSCRIBERS]>;

// type IntMap<K, V> = FxHashMap<K, V, BuildNoHashHasher<K>>;

pub struct InnerBroker {
    pub subscribers: FxHashMap<Subscriber, SMtDeque>,
    pub subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE],
}

impl Default for InnerBroker {
    fn default() -> Self {
        let subscribers: FxHashMap<Subscriber, SMtDeque> = FxHashMap::default();
        let subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE] = Default::default();
        Self { subscribers, subscribed_msgs }
    }
}

pub struct InnerBroker2 {
    pub subscribers: IntMap<Subscriber2, SMtDeque>, //FxHashMap<Subscriber, Channel>,
    pub subscribed_msgs: [MsgSubscribedList2; COUNT_EVENT_TYPE],
}

impl Default for InnerBroker2 {
    fn default() -> Self {
        let subscribers: IntMap<Subscriber2, SMtDeque> = IntMap::default(); //FxHashMap<Subscriber, Channel> = FxHashMap::default();
        let subscribed_msgs: [MsgSubscribedList2; COUNT_EVENT_TYPE] = Default::default();
        Self { subscribers, subscribed_msgs }
    }
}

#[repr(u8)]
#[derive(Clone, Eq, PartialEq)]
pub enum Subscriber1 {
    Main = 0,
    Mqtt = 1,
    Test = 2,
    Null = 3,
}
pub struct InnerBroker1 {
    pub subscribers: [(Subscriber1, SMtDeque); MAX_SUBSCRIBERS],
    pub subscribed_msgs: [MsgSubscribedList1; COUNT_EVENT_TYPE],
}

impl Default for InnerBroker1 {
    fn default() -> Self {
        let subscribers: [(Subscriber1, SMtDeque); MAX_SUBSCRIBERS] =
            [(Subscriber1::Null, Arc::new(MtDeque::new())), (Subscriber1::Null, Arc::new(MtDeque::new())), (Subscriber1::Null, Arc::new(MtDeque::new()))];
        let subscribed_msgs: [MsgSubscribedList1; COUNT_EVENT_TYPE] = Default::default();
        Self { subscribers, subscribed_msgs }
    }
}

pub fn process_msg_vbase(msg: &Box<IntMessage>, inner_broker: &InnerBroker) {
    let index = msg.data.tipo();
    let no_subscriber_string: String = msg.data.to_string();

    log_info!(info_broker_gen(&no_subscriber_string, &msg.data.to_string()));

    let subscritores = &inner_broker.subscribed_msgs[index];

    // para todos os subscritores desta mensagem, vamos enviá-la
    for subs in subscritores.iter() {
        let _channel = inner_broker.subscribers.get(subs).unwrap();
        //simula send
        thread::sleep(Duration::from_nanos(100));
    }
}

pub fn process_msg_vbase_e(msg: &Box<IntMessage>, inner_broker: &InnerBroker2) {
    let index = msg.data.tipo();
    let no_subscriber_string: String = msg.data.to_string();

    log_info!(info_broker_gen(&no_subscriber_string, &msg.data.to_string()));

    let subscritores = &inner_broker.subscribed_msgs[index];

    // para todos os subscritores desta mensagem, vamos enviá-la
    for subs in subscritores.iter() {
        let _channel = inner_broker.subscribers.get(subs).unwrap();
        //simula send
        thread::sleep(Duration::from_nanos(100));
    }
}

pub fn process_msg_v1(msg: &Box<IntMessage>, inner_broker: &InnerBroker1) {
    let index = msg.data.tipo();
    let no_subscriber_string: String = msg.data.to_string();

    log_info!(info_broker_gen(&no_subscriber_string, &msg.data.to_string()));

    let subscritores = &inner_broker.subscribed_msgs[index];

    // para todos os subscritores desta mensagem, vamos enviá-la
    for subs in subscritores.iter() {
        if *subs != Subscriber1::Null {
            let _channel = &inner_broker.subscribers[(subs.clone() as u8) as usize].1;

            //simula send
            thread::sleep(Duration::from_nanos(100));
        }
    }
}

pub fn broker_process(d: &mut Criterion) {
    let mut c = d.benchmark_group("broker_process");

    let mut inner_broker = InnerBroker::default();
    let mut inner_broker1 = InnerBroker1::default();
    let mut inner_broker2 = InnerBroker2::default();
    let msg = Box::new(IntMessage::build(MsgData::CycleAdded, CtrlTime::sys_time()));

    c.bench_function("process_msg_vbase", |b| b.iter(|| black_box(process_msg_vbase(&msg, &inner_broker))));
    c.bench_function("process_msg_vbase_e", |b| b.iter(|| black_box(process_msg_vbase_e(&msg, &inner_broker2))));
    c.bench_function("process_msg_v1", |b| b.iter(|| black_box(process_msg_v1(&msg, &inner_broker1))));

    inner_broker.subscribers.insert(Subscriber::Main, Arc::new(MtDeque::new()));
    inner_broker.subscribers.insert(Subscriber::Mqtt, Arc::new(MtDeque::new()));

    inner_broker1.subscribers[0].0 = Subscriber1::Main;
    inner_broker1.subscribers[1].0 = Subscriber1::Mqtt;

    inner_broker2.subscribers.insert(Subscriber2::Main, Arc::new(MtDeque::new()));
    inner_broker2.subscribers.insert(Subscriber2::Mqtt, Arc::new(MtDeque::new()));
    c.bench_function("process_msg_vbase_with_data", |b| b.iter(|| black_box(process_msg_vbase(&msg, &inner_broker))));
    c.bench_function("process_msg_vbase_e_with_data", |b| b.iter(|| black_box(process_msg_vbase_e(&msg, &inner_broker2))));
    c.bench_function("process_msg_v1_with_data", |b| b.iter(|| black_box(process_msg_v1(&msg, &inner_broker1))));
    c.finish();
}
