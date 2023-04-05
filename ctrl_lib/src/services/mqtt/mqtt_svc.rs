/// SPRINT SECURITY - maybe be gather some statistics to identify potential DoS threats
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use mqtt::{QOS_0, ReasonCode};
use paho_mqtt as mqtt;
use string_concat::*;

use crate::app_time::ctrl_time::*;
use crate::config::mqtt_config::*;
use crate::data_structs::concurrent_queue::*;
use crate::data_structs::msgs::{topic::*, int_message::*, connection::*};
use crate::services::{msg_broker::subscriber::*, msg_broker::msg_brkr_svc::*, mqtt::paho::*};
use crate::utils::*;
use crate::{log_error, logger::*, log_info, lib_serde::Json};
use ctrl_prelude::{error::*, globals::*, string_resources::*};

const SECONDS_CHECK_INTERVAL: u64 = 1;

/// Dimension = 16 bytes
pub struct MQTTService {
    pub_queue: SMtDeque,
    msg_broker: SMsgBrkr,
}

impl MQTTService {
    #[inline]
    pub fn new(msg_broker: SMsgBrkr) -> Self {
        let pub_queue = Arc::new(MtDeque::new());
        Self { 
            pub_queue, 
            msg_broker, 
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn start(&mut self) -> JoinHandle<()>{
        let msg_broker = self.msg_broker.clone();
        let pub_queue = self.pub_queue.clone();

        let builder = thread::Builder::new().name(MQTT_SERVICE_THREAD.to_owned()).stack_size(17 * STACK_SIZE_UNIT); // estava em 20 ok, 10 nok, 17 ok
        
        builder.spawn(move || {
            #[cfg(debug_assertions)]
            { unsafe { THREAD_COUNT += 1; } }
            msg_broker.register_in_broker(Subscriber::Mqtt, pub_queue.clone());

            let config = MQTTConfig::new();
            let create_opts: mqtt::CreateOptions = mqtt::CreateOptionsBuilder::new()
                                    .user_data(Box::new(msg_broker.clone()))
                                    .server_uri(string_concat!("tcp://", config.broker_address, ":", config.broker_port.to_string()))
                                    .client_id(string_concat!(config.client_id,"_pub"))
                                    .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
                                    .finalize();
            let cli_result = mqtt::AsyncClient::new(create_opts);

            match cli_result{
                Ok(mut cli_mqtt) => {
                    msg_broker.subscribe(MsgType::MessageOut, Subscriber::Mqtt);

                    cli_mqtt.set_connected_callback(on_connection);
                    cli_mqtt.set_connection_lost_callback(on_connection_lost);
                    cli_mqtt.set_message_callback(on_message);
                    

                    let lwt_msg = Connection::new_out(false, CtrlTime::sys_time());
                    let lwt = mqtt::Message::new_retained(Topic::SERVER_CONNECTION.to_string(), lwt_msg.json().unwrap(), QOS_0);
            
                    let conn = mqtt::ConnectOptionsBuilder::new()
                        .keep_alive_interval(Duration::from_secs(120))
                        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
                        .clean_session(true)
                        .will_message(lwt)
                        .connect_timeout(Duration::new(5, 0))
                        // retry during 12 hours
                        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(33200))
                        .finalize();
            
                    let _conn_token = cli_mqtt.connect_with_callbacks(conn, on_connect_success, on_connect_failure);
            
                    let interval = SECONDS_CHECK_INTERVAL * GIGA_U;
                    loop {
                        let (opt_i_msg, _is_time_out, is_shutdown) = pub_queue.recv_timeout(get_deadline_instant(interval));
                        if let Some(msg) = opt_i_msg {
                            // only care for send msg events
                            if let MsgData::MessageOut(msg_out) = &msg.data {
                                // only send if we are connected to the broker, otherwise, eat the msg and do nothing
                                if cli_mqtt.is_connected() {
                                    // 2023-1-23 with usage and debug, clocks (server & client) have to be in sync
                                    send(msg_out, &cli_mqtt);
                                }
                            }
                        } else if is_shutdown {
                            break;
                        }
                        // if not shutdown, and msg == None =>  timeout
                        // by mqtt machinery it will retry automatically on/if disconnection 
                    } //loop end
                    
                    if cli_mqtt.is_connected() {
                        send(&lwt_msg, &cli_mqtt);
                        if !{unsafe{TESTING}}{
                            cli_mqtt.unsubscribe_many(SUBS);
                        }else{
                            cli_mqtt.unsubscribe_many(SUBS_TEST);
                        }
                        let opts = mqtt::DisconnectOptionsBuilder::new().reason_code(ReasonCode::AdministrativeAction).finalize();
                        _ = cli_mqtt.disconnect(opts).wait();
                    }
                    log_info!(INFO_MQTT_ENDING);
                },
                Err(e) => log_error!(err_mqtt_int_clnt_creation(&build_error(&e))),
            }

            #[cfg(debug_assertions)]
            { unsafe { THREAD_COUNT -= 1; } }
        }).unwrap()
    }

    #[inline]
    pub fn terminate(&self) {
        // println!("nr de mensagens a processar no terminate: {}", self.pub_queue.len());
        self.pub_queue.terminate();
    }

}
