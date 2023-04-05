//! Message Broker
//!
//! We have one channel per msg type 
//!
//! Subscribers can subscribe any msg type
//!
//! Each msg type provides a channel (queue) for communication
//!
//! Is subscriber responsability to listen on the provided channel
//!
//! Central management point for all the msgs in the application
//!
//! Receives all the msgs and dispatch them to the subscribers
//! 
//! In this way is also possible to have a centralized msg log 
//!
//! Broker Manager have is own thread, so publishers threads don't have to wait
//! On msg publish, thread wakes up and do stuff, without holding the publisher thread
//!  - mqtt (blocking ons sockets)
//!  - web - blocks on web response, and sometime, wait for broker msgs
//!  - io (blocking on external io)
//!  - sensors/actuators - similar to io
//!  - commands, either from web, or from mqtt, or from program (shutdown, etc.) or ctrl c/interrupt on keyboard
//!  - inknown errors :-)
//!

use std::{sync::Arc, thread};

use arrayvec::ArrayVec;
use parking_lot::{RwLock, lock_api::RwLockWriteGuard};
// pub use no_deadlocks::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_hash::FxHashMap;

use crate::app_time::ctrl_time::*;
use crate::data_structs::concurrent_queue::*;
use crate::data_structs::msgs::{ext_message::*, int_message::*, log_error::*};
use crate::services::msg_broker::subscriber::*;
use crate::{log_debug, log_info, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};

#[cfg(debug_assertions)]
use crate::log_warn;
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;

type MsgSubscribedList = ArrayVec<Subscriber, MAX_SUBSCRIBERS>;

///Dimension = 8
pub type SMsgBrkr = Arc<MsgBrkr>;

/// Dimension = 16
pub struct MsgBrkr {
    pub inner: Arc<RwLock<InnerBroker>>,
    pub evt_in: SMtDeque,
}

impl MsgBrkr {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(InnerBroker::default()));
        MsgBrkr {
            inner,
            evt_in: Arc::new(MtDeque::new()),
        }
    }

    #[inline]
    pub fn register_in_broker(&self, subscriber: Subscriber, out_queue: SMtDeque) {
        let o_channel: Option<SMtDeque>;
        {
            // o_channel = self.inner.write().unwrap().subscribers.insert(subscriber.clone(), out_queue);
            o_channel = self.inner.write().subscribers.insert(subscriber, out_queue);
        };
        if let Some(_channel) = o_channel {
            // already registered in broker
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_dup_subs(subscriber.to_string()));
        } else {
            // just to evaluate cardinality to maybe change capacity reservation
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_nr_of_subs(self.inner.read().subscribers.len()));
            // log_info!(dbg_brkr_nr_of_subs(self.inner.read().unwrap().subscribers.len()));
        }
    }

    /// Should be called only after registration
    #[inline]
    pub fn unregister_in_broker(&self, subscriber: Subscriber) {
        let _o_channel: Option<SMtDeque>;
        {
            _o_channel = self.inner.write().subscribers.remove(&subscriber);
        };
        #[cfg(debug_assertions)]
        self.log_debug_info(_o_channel, subscriber);
    }

    #[inline]
    pub fn unregister_and_unsubscribe(&self, subscriber: Subscriber) {
        let _o_channel: Option<SMtDeque>;
        {
            self.unsubscribe_all(subscriber);
            _o_channel = self.inner.write().subscribers.remove(&subscriber);
        };
        #[cfg(debug_assertions)]
        self.log_debug_info(_o_channel, subscriber);
    }

    #[cfg(debug_assertions)]
    #[inline]
    fn log_debug_info(&self, o_channel: Option<Arc<MtDeque<IntMessage>>>, subscriber: Subscriber) {
        if let Some(_channel) = o_channel {
            // existing key
            #[cfg(debug_assertions)]
            dbg_brkr_nr_of_subs(self.inner.read().subscribers.len());
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_dup_subs(subscriber.to_string()));
        } else {
            // inexistent key
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_dup_subs(subscriber.to_string()));
        }
    }

    /// call after unsubscribe
    #[inline]
    pub fn unsubscribe(&self, msg_type: MsgType, subscriber: Subscriber) -> bool {
        let index = msg_type as usize;
        let mut result: bool = false;
        {
            let mut guard = self.inner.write();
            // let mut guard = self.inner.write().unwrap();
            if guard.subscribers.get(&subscriber).is_some() {
                remove_unsubscribed(&mut guard, index, subscriber, &mut result);
            }
        }
        //temporarly to evaluate cardinality and reserve capacity
        #[cfg(debug_assertions)]
        log_info!(dbg_brkr_nr_of_msgs_subs(&msg_type.to_string(), self.inner.read().subscribed_msgs[index].len()));
        // log_info!(dbg_brkr_nr_of_msgs_subs(&msg_type.to_string(), self.inner.read().unwrap().subscribed_msgs[index].len()));
        result
    }

    /// call after unsubscribe
    #[inline]
    pub fn unsubscribe_all(&self, subscriber: Subscriber) -> bool {
        let mut result: bool = false;
        {
            let mut guard = self.inner.write();
            // let mut guard = self.inner.write().unwrap();
            if guard.subscribers.get(&subscriber).is_some() {
                for j in 0..COUNT_EVENT_TYPE {
                    remove_unsubscribed(&mut guard, j, subscriber, &mut result);
                }
            }
        }
        result
    }

    #[inline]
    pub fn subscribe(&self, msg_type: MsgType, subscriber: Subscriber) -> bool {
        let index = msg_type as usize;
        let mut result: bool = false;
        {
            let mut guard = self.inner.write();
            // let mut guard = self.inner.write().unwrap();
            if guard.subscribers.get(&subscriber).is_some() {
                guard.subscribed_msgs[index].push(subscriber);
                result = true;
            }
        }
        result
    }

    #[inline]
    #[rustfmt::skip]
    pub fn get_msg_brkr_handle(&self) -> SMtDeque { self.evt_in.clone() }

    #[inline]
    pub fn reg_int_msg(&self, msg_data: MsgData, time: CtrlTime) {
        //temporarly to evaluate cardinality and reserve capacity
        let l = self.evt_in.len();
        if l > 0 {
            #[cfg(debug_assertions)]
            log_warn!(dbg_brkr_nr_of_msg_in(l));
            log_debug!(msg_data.to_string());
        }
        let index = msg_data.tipo();
        let have_subscriber: bool;
        {
            let read_guard = self.inner.read();
            // let read_guard = self.inner.read().unwrap(); // isto está aqui para se for preciso despistar deadlocks
            let subscritores = &read_guard.subscribed_msgs[index];
            have_subscriber = !subscritores.is_empty();
        }
        // only send msg with at least one subscriber
        if have_subscriber {
            self.evt_in.send(IntMessage::build(msg_data, time));
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn snd_error_to_client(&self, err_msg: &str) {
        self.reg_int_msg(MsgData::ClientError(LogError { header: None, error: err_msg.to_owned(), }), CtrlTime::sys_time());
    }

    #[inline]
    pub fn snd_shut_down(&self) {
        let time = CtrlTime::sys_time();
        self.reg_int_msg(MsgData::ShutDown(time), CtrlTime::sys_time());
    }

    #[inline]
    pub fn terminate(&self) {
        self.evt_in.terminate();
    }

    #[inline]
    pub fn snd_status_changed(&self, time: CtrlTime) {
        self.reg_int_msg(MsgData::StateChanged, time);
    }

    #[inline]
    pub fn snd_ext_msg(&self, message: ExtMsgOut) {
        self.reg_int_msg(MsgData::MessageOut(message), CtrlTime::sys_time());
    }

    #[inline]
    #[rustfmt::skip]
    pub fn start(&self) -> thread::JoinHandle<()> {
        let builder = thread::Builder::new().name(EVBR_SERVICE_THREAD.to_owned()).stack_size(18 * STACK_SIZE_UNIT);
        let inner_broker = self.inner.clone();
        let broker_rcv = self.evt_in.clone();

        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                log_info!(INFO_BRKR_THREAD_START);
                let mut opt_msg : Option<IntMessage>;
                loop{
                    opt_msg = broker_rcv.recv();
                    if let Some(msg) = opt_msg{
                        // in principle no need to sync with new_day_and_db_mnt_sync()
                        // Only the subscribers, that will be doing stuff with the message may need to sync
                        // in thesis, only weather, water machine, and scenes make db access
                        // new_day_and_db_mnt_sync();
                        process_msg(Box::new(msg), &inner_broker);
                    } else {
                        break; // if terminating, exit
                    }
                } //lopp end
                log_info!(INFO_BRKR_THREAD_STOP);
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
            })
            .unwrap()
    }
}

fn remove_unsubscribed(guard: &mut RwLockWriteGuard<parking_lot::RawRwLock, InnerBroker>, index: usize, subscriber: Subscriber, result: &mut bool) {
    for i in 0..guard.subscribed_msgs[index].len() {
        if guard.subscribed_msgs[index][i] == subscriber {
            guard.subscribed_msgs[index].remove(i);
            *result = true;
            break;
        }
    }
}

// function extracted from main loop to easy mesasure performance and alocations
#[inline]
pub fn process_msg(msg: Box<IntMessage>, inner_broker: &Arc<RwLock<InnerBroker>>) {
    let index = msg.data.tipo();
    let no_subscriber_string: String = msg.data.to_string();

    log_debug!(info_broker_gen(&no_subscriber_string, &msg.data.to_string()));

    let mut subs_queue: &SMtDeque;
    let read_guard = inner_broker.read();
    // let read_guard = inner_broker.read().unwrap();
    let subscritores = &read_guard.subscribed_msgs[index];

    // send msgs to all subscribers.
    for subs in subscritores.iter() {
        subs_queue = read_guard.subscribers.get(subs).unwrap();
        // if *subs == Subscriber::Mqtt {
        //     println!("vai enviar msg para o mqtt")
        // }
        subs_queue.send(*msg.clone());
        // if *subs == Subscriber::Mqtt {
        //     println!("enviou msg para o mqtt")
        // }
    }
    #[cfg(debug_assertions)]
    // msg with no subscriber.  Debug info.  Maybe eliminate for production
    if subscritores.is_empty() {
        log_debug!(dbg_brkr_no_sbs(&no_subscriber_string, &msg.data.to_string()));
    }
}

/// Dimension = 168
pub struct InnerBroker {
    /// subscribers list
    pub subscribers: FxHashMap<Subscriber, SMtDeque>,
    /// msgs list, with the subsbcribers of each msg
    pub subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE],
}

impl Default for InnerBroker {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self {
        let subscribers: FxHashMap<Subscriber, SMtDeque> = FxHashMap::default();
        let subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE] = Default::default();
        Self { subscribers, subscribed_msgs, }
    }
}
