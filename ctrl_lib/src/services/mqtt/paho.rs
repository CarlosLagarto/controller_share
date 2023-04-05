use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use lazy_static::*;
use paho_mqtt as mqtt;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use string_concat::*;

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{connection::*, ext_message::*, int_message::*, topic::*};
use crate::services::{mqtt::*, msg_broker::msg_brkr_svc::*};
use crate::utils::TESTING;
use crate::{lib_serde::*, log_debug, log_error, log_info, log_warn, logger::*};
use ctrl_prelude::{error::*, globals::*, string_resources::*};

const MAX_CACHE_SIZE: u8 = 100;

lazy_static! {
    pub static ref MSGS_CACHE: Mutex<CacheFx> = Mutex::new(CacheFx::with_capacity(MAX_CACHE_SIZE));
    pub static ref CONNECTED_CLIENTS: Mutex<FxHashMap<String, bool>> =Mutex::new(FxHashMap::default()); // key = client_id, bool = conected or not conected
    pub static ref LAST_HB: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    pub static ref IS_CONNECTED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// Callback for a successful connection to the broker.
// We subscribe to the topic(s) we want here.
#[inline]
pub fn on_connect_success(cli: &mqtt::AsyncClient, msgid: u16) {
    log_info!(string_concat!(INFO_MQTT_CONNECTED_1P, ToString::to_string(&msgid)));
    if !unsafe { TESTING } {
        cli.subscribe_many(SUBS, QOS_TOPICS);
    } else {
        cli.subscribe_many(SUBS_TEST, QOS_TOPICS);
    }
    send(&Connection::new_out(true, CtrlTime::sys_time()), cli);
}

// Callback for a failed attempt to connect to the server.
#[inline]
pub fn on_connect_failure(_cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
    log_warn!(err_mqtt_refused_connection(&rc.to_string(), ""));
    IS_CONNECTED.store(false, Ordering::Relaxed);
}

#[inline]
pub fn on_connection(_cli: &mqtt::AsyncClient) {
    IS_CONNECTED.store(true, Ordering::Relaxed);
}

#[inline]
pub fn on_connection_lost(_cli: &mqtt::AsyncClient) {
    IS_CONNECTED.store(false, Ordering::Relaxed);
    if unsafe { SHUTTING_DOWN } {
        log_info!(INFO_MQTT_DISCONNECTED);
    }
}

#[inline]
pub fn send(msg_out: &ExtMsgOut, cli_mqtt: &mqtt::AsyncClient) {
    match msg_out.json() {
        Ok(msg) => {
            println!("send: {}", &msg);
            let new_msg: mqtt::Message;
            if let ExtMsgOut::Connection(conn) = msg_out {
                new_msg = mqtt::Message::new_retained(conn.header.as_ref().unwrap().topic.to_string(), msg, mqtt::QOS_0);
            } else {
                new_msg = mqtt::Message::new(msg_out.header().unwrap().topic.to_string(), msg, mqtt::QOS_1);
            };
            let _delivery_token = cli_mqtt.publish(new_msg);
            log_debug!(DBG_MQTT_MSG_PUBLISHED)
        }
        Err(e) => {
            log_error!(err_snd_mqtt_msg(&build_error(&e)));
        }
    }
}

#[inline]
pub fn on_message(mqtt_cli: &mqtt::AsyncClient, mqtt_msg: Option<mqtt::Message>) {
    if let Some(cli_msg) = mqtt_msg {
        let topic_str = cli_msg.topic();
        let payload = cli_msg.payload_str();
        println!("recebemos msg do mosquitto: {}", &payload);

        let o_msg_broker = mqtt_cli.user_data().unwrap();
        let msg_broker: &SMsgBrkr = o_msg_broker.downcast_ref::<SMsgBrkr>().unwrap();

        let topic = Topic::from_string(topic_str);
        let time = CtrlTime::sys_time();
        if topic == Topic::SHELLIES {
            // shellies periodically (30 secs) send stuff
            msg_broker.reg_int_msg(MsgData::Shellies(String::from(topic_str), String::from(payload)), time);
        } else {
            match ExtMsgIn::new(topic, &payload) {
                Ok(msg) => {
                    
                    let header = msg.header().unwrap();
                    // any message act as client HB.
                    // HB msg only is relevant if the client is without iteraction for some time
                    LAST_HB.store(time.0, Ordering::Relaxed);
                    if header.topic != Topic::CLIENT_CONNECTION {
                        // validate cache to evaluate weird stuff, avoid command repetion, idempotence&  etcs
                        let cache_result;
                        if let Some(uuid) = header.uuid.as_ref() {
                            cache_result = MSGS_CACHE.lock().push(uuid);
                            if cache_result != CacheResult::Duplicate {
                                // all msgs go to broker.
                                msg_broker.reg_int_msg(MsgData::MessageIn(msg), time);
                            } else {
                                // msg duplicada
                                log_warn!(warn_mqtt_dup_msg(&header.client_id, &header.topic.to_string()));
                            }
                        } else {
                            log_error!("Esperava-se que a mensagem tivesse uuid e não têm");
                        }
                    } else if let ExtMsgIn::Connection(connection) = msg {
                        // we may have more than one client, so we have a hash table with the connected clients (or history of connected clients)
                        let client_id = connection.header.as_ref().unwrap().client_id.to_owned();
                        if connection.status == CONNECTION::ONLINE.to_string() {
                            CONNECTED_CLIENTS.lock().insert(client_id, true);
                        } else {
                            CONNECTED_CLIENTS.lock().insert(client_id, false);
                        }
                        // msg_broker.reg_int_msg(MsgData::MessageOut(Connection::new_out(true, time)), time);
                    }
                }
                Err(err) => {
                    let msg = err_unknown_mqtt_msg(&err.to_string(), topic_str, &payload);
                    log_error!(&msg);
                    println!("mensagem com erro:\n{}", &payload);
                    msg_broker.snd_error_to_client(&msg);
                }
            }
        }
    }; //If None, do nothing.  It is the mqtt library shutting down the consumers
}
