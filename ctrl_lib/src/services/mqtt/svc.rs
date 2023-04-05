/// SPRINT SECURITY - pensar aqui em estatisticas para identificar nr de chamadas (DoS) acima do razoável que pode indicar mais alguém a aceder á máquina
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ctrl_prelude::error::build_error;
use paho_mqtt as mqtt;
use parking_lot::RwLock;
use string_concat::*;
// pub use no_deadlocks::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::config::mqtt_config::*;
use crate::data_structs::msgs::{connection::*, ext_message::*, int_message::*, topic::*};
use crate::data_structs::{channel::*, timer_signals::*};
use crate::lib_serde::Json;
use crate::services::{msg_broker::subscriber::*, msg_broker::svc::*, mqtt::*};
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{log_debug, log_error, log_info, log_warn, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};


const QOS_1: i32 = 1;
const QOS_0: i32 = 0;
const MAX_CACHE_SIZE: u8 = 100;

const COUNT_SUBSCRIPTIONS: usize = 11;

type InnerMqtt = Arc<RwLock<MQTT>>;

/// Dimensão = 48 bytes
pub struct MQTTService {
    ctrl_channel: Channel,
    mqtt: InnerMqtt,
    msg_broker: SMsgBrkr,
}

impl MQTTService {
    #[inline]
    #[rustfmt::skip]
    pub fn new(msg_broker: SMsgBrkr) -> Self {
        let ctrl_channel = msg_broker.register_in_broker(Subscriber::Mqtt);
        let mqtt = Arc::new(RwLock::new(MQTT::new()));
        {
            let mut mqtt_lock = mqtt.write();
            // let mut mqtt_lock = mqtt.write().unwrap();
            mqtt_lock.mqtt_tx = Some(ctrl_channel.tx.clone());
            mqtt_lock.msg_broker = Some(msg_broker.clone());
        }
        Self { ctrl_channel, mqtt, msg_broker, }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn start(&mut self) -> thread::JoinHandle<()> {
        let i_mqtt = self.mqtt.clone();
        let msg_broker = self.msg_broker.clone();
        let ctrl_channel = self.ctrl_channel.clone();

        let builder = thread::Builder::new().name(MQTT_SERVICE_THREAD.to_owned()).stack_size(17 * STACK_SIZE_UNIT); // estava em 20 ok, 10 nok, 15 nok, 16 nok, 17 ok
        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                let create_opts: mqtt::CreateOptions;
                {
                    let mut mqtt_lock = i_mqtt.write();
                    // let mut mqtt_lock = i_mqtt.write().unwrap();
                    let _wait_mqtt_chk_interval = mqtt_lock.config.wait_mqtt_chk_interval as u16;
                    mqtt_lock.mqtt_check_task = Some(Schedule::build_run_forever(CtrlTime::sim_time(), _wait_mqtt_chk_interval, ScheduleRepeatUnit::Seconds));
                    log_info!(INFO_MQTT_STARTING);

                    create_opts = mqtt::CreateOptionsBuilder::new()
                        .user_data(Box::new(i_mqtt.clone()))
                        .server_uri(string_concat!(&mqtt_lock.config.broker_address, ":", mqtt_lock.config.broker_port.to_string()))
                        .client_id(mqtt_lock.config.client_id.clone())
                        .finalize();
                }
                let cli = mqtt::AsyncClient::new(create_opts);
                let cli_temp: mqtt::AsyncClient;
                if let Ok(_cli) = cli {
                    {
                        let mut mqtt_lock = i_mqtt.write();
                        // let mut mqtt_lock = i_mqtt.write().unwrap();
                        mqtt_lock.client = Some(_cli);
                        cli_temp = mqtt_lock.client.to_owned().unwrap();
                        cli_temp.set_connected_callback(on_connect);
                        cli_temp.set_connection_lost_callback(on_connection_lost);
                        cli_temp.set_disconnected_callback(on_disconnect);
                        cli_temp.set_message_callback(on_message);
                        mqtt_lock.connect();
                    }

                    msg_broker.subscribe(MsgType::MessageOut, Subscriber::Mqtt);
                    msg_broker.subscribe(MsgType::TimeSignal, Subscriber::Mqtt);
                    msg_broker.subscribe(MsgType::StopMQTT, Subscriber::Mqtt);
                    loop {
                        let msg = ctrl_channel.rx.recv().unwrap();
                        // A coordenação caso se esteja a processar o novo dia ou a manter a base de dados é da responsabilidade
                        // de quem chama acede á base de dados.  O MQTT não iterage diretamente com BD pelo que não precisa.
                        // event_manager_wait();
                        match msg.data {
                            //só interessa os eventos para enviar mensagens.
                            MsgData::MessageOut(msg_out) => {
                                let mqtt_lock = i_mqtt.write();
                                // let mqtt_lock = i_mqtt.write().unwrap();
                                // só se pública se houver broker, senão, consome-se a msg mas não se faz nada
                                if mqtt_lock.connected_to_broker {
                                    mqtt_lock.send(&msg_out);
                                }
                            }
                            MsgData::TimeSignal(TimerSignal::Timer(time)) => {
                                // religa automticamente caso a conexão caia.
                                // ? REVIEW: - temos que avaliar que se isto for constante, só devemos tentar x vezes, e depois deixar de
                                // andar a martelar a coisa.  Algum problema existe que terá que ser resolvido
                                // se for transiente....prever funcionalidade para relançar por um canal qualquer o timer a ver se já funciona
                                let mut mqtt_lock = i_mqtt.write();
                                // let mut mqtt_lock = i_mqtt.write().unwrap();
                                if mqtt_lock.mqtt_check_task.as_ref().unwrap().is_time_to_run(time) {
                                    log_debug!(DBG_MQTT_RETRYING_CONNECT);
                                    mqtt_lock.connect();
                                    //avançamos para o próximo evento
                                    _ = mqtt_lock.mqtt_check_task.as_mut().unwrap().set_next_event().map_err(|e| log_error!(build_error(&e)));
                                }
                            }
                            MsgData::StopMQTT => {
                                i_mqtt.write().terminate();
                                // i_mqtt.write().unwrap().terminate();
                                break;
                            } //a thread termina
                            _ => (), //ignoramos tudo o resto.
                        }
                    } //fim do loop
                      // cli.disconnect(None); //isto desapareceu da LIB do mqtt.
                      // a 22/Jan/2022 está a desligar sem temas.
                    log_info!(INFO_MQTT_ENDING);
                } else if let Err(e) = cli {
                    //Se não conseguimos criar o cliente temos um RETURN como guarda
                    log_error!(err_mqtt_int_clnt_creation(&build_error(&e)));
                }
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
            })
            .unwrap() //end spawn
    }

    #[inline]
    pub fn terminate(&self) {
        let event = IntMessage::build(MsgData::StopMQTT, CtrlTime::sim_time(), DESC_MQTT_TERMINATE_MSG);
        self.ctrl_channel.tx.send(event).unwrap();
    }

    #[inline]
    pub fn connected_to_broker(&self) -> bool {
        self.mqtt.read().connected_to_broker
        // self.mqtt.read().unwrap().connected_to_broker
    }
}

/// Dimensão = 232 bytes
#[allow(clippy::upper_case_acronyms)]
pub struct MQTT {
    client: Option<mqtt::AsyncClient>,
    config: MQTTConfig,
    msgs_cache: CacheFx,
    mqtt_check_task: Option<Schedule>,
    msg_broker: Option<SMsgBrkr>,
    pub mqtt_tx: Option<EventSender>,
    subscriptions: [Topic; COUNT_SUBSCRIPTIONS],
    last_heart_beat_time: CtrlTime,
    connected_to_broker: bool,
    is_shutting_down: bool,
}

impl MQTT {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            mqtt_tx: None,
            client: None,
            connected_to_broker: false,
            mqtt_check_task: None,
            config: MQTTConfig::new(),
            last_heart_beat_time: CtrlTime(0),
            is_shutting_down: false,
            msg_broker: None,
            subscriptions: [
                Topic::CTS_STOP_SECTOR,
                Topic::CTS_STOP_CYCLE,
                Topic::CTS_STATUS_CHANGE_MODE,
                Topic::CTS_FORCE_SECTOR,
                Topic::CTS_FORCE_CYCLE,
                Topic::CTS_STATUS_SHUTDOWN,
                Topic::CTS_SYNC_DB,
                Topic::CTS_GET_FULLDB,
                Topic::CTS_STATUS_RESTART,
                Topic::CLIENT_CONNECTION,
                Topic::CTS_GET_WEATHER_HIST,
            ],
            // SPRINT SENSORS - Sensors - Topic::DEVICE_1_CONNECTION]
            msgs_cache: CacheFx::with_capacity(MAX_CACHE_SIZE),
        }
    }

    #[inline]
    fn connect(&mut self) {
        self.connected_to_broker = false;
        let lwt_msg = Connection::new_out(false, CtrlTime::sim_time()).json().unwrap();
        let lwt = mqtt::Message::new_retained(Topic::SERVER_CONNECTION.to_string(), lwt_msg, QOS_0);

        let conn = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(true)
            .will_message(lwt)
            .finalize();
        let cli = self.client.to_owned().unwrap();
        _ = cli.connect_with_callbacks(conn, on_connect_success, on_connect_failure);
    }

    #[inline]
    fn send(&self, message: &ExtMsg) {
        match message.json() {
            Ok(msg) => {
                let new_msg: mqtt::Message = if !(message.topic == Topic::SERVER_CONNECTION) {
                    mqtt::Message::new_retained(message.topic.to_string(), msg, QOS_0)
                } else {
                    mqtt::Message::new(message.topic.to_string(), msg, QOS_1)
                };
                let mqtt_cli = self.client.to_owned().unwrap();
                let res: mqtt::DeliveryToken = mqtt_cli.publish(new_msg);
                res.wait_for(Duration::from_secs(5)).map_or_else(|e| log_warn!(e.to_string()), |_| log_debug!(DBG_MQTT_MSG_PUBLISHED))
            }
            Err(e) => {
                log_error!(err_snd_mqtt_msg(&build_error(&e)));
            }
        }
    }

    #[inline]
    pub fn terminate(&mut self) {
        // force offline msg - needed because the lastwill message is only sent on abnormal desconnect
        self.send(&Connection::new_out(false, CtrlTime::sim_time()));
        // disconnect
        self.connected_to_broker = false;
        self.is_shutting_down = true;
    }
}

#[inline]
fn on_connect_success(cli: &mqtt::AsyncClient, msgid: u16) {
    let o_mqtt = cli.user_data().unwrap();
    if let Some(i_mqtt) = o_mqtt.downcast_ref::<InnerMqtt>() {
        set_connected(i_mqtt, true);
    }
    log_info!(string_concat!(INFO_MQTT_CONNECTED_1P, ToString::to_string(&msgid)));
}

#[inline]
fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
    let o_mqtt = cli.user_data().unwrap();
    if let Some(i_mqtt) = o_mqtt.downcast_ref::<InnerMqtt>() {
        set_connected(i_mqtt, false);
    }
    log_error!(err_mqtt_refused_connection(&rc.to_string(), ""));
}

// client.subscribe("$SYS/#")
#[inline]
fn on_connect(cli: &mqtt::AsyncClient) {
    let o_mqtt = cli.user_data().unwrap();
    if let Some(i_mqtt) = o_mqtt.downcast_ref::<InnerMqtt>() {
        let mut mqtt_lock = i_mqtt.write();
        // let mut mqtt_lock = i_mqtt.write().unwrap();
        mqtt_lock.mqtt_check_task.as_mut().unwrap().start = CtrlTime(CtrlTime::MAX);
        for topic in mqtt_lock.subscriptions.iter() {
            _ = cli.subscribe(topic.to_string(), QOS_1);
            // NOTE esta versao da lib perdeu o callback para dar info de que a subscrição correu ok.
        }
    }
    if let Some(i_mqtt) = o_mqtt.downcast_ref::<InnerMqtt>() {
        let mqtt_lock = i_mqtt.read();
        // let mqtt_lock = i_mqtt.read().unwrap();
        let time = CtrlTime::sim_time();
        mqtt_lock.msg_broker.as_ref().unwrap().reg_int_msg(MsgData::MessageOut(Connection::new_out(true, time)), time, DESC_MQTT_HB_MSG_SRVR_CLNT);
    }
}

#[inline]
fn on_disconnect(cli: &mqtt::AsyncClient, _prop: mqtt::Properties, _rc: mqtt::ReasonCode) {
    let o_mqtt = cli.user_data().unwrap();
    if let Some(i_mqtt) = o_mqtt.downcast_ref::<InnerMqtt>() {
        set_connected(i_mqtt, false);
    }
}

#[inline]
fn set_connected(i_mqtt: &Arc<RwLock<MQTT>>, flag: bool) {
    let mut guard = i_mqtt.write();
    // let mut guard = i_mqtt.write().unwrap();
    guard.connected_to_broker = flag;
}

#[inline]
fn on_connection_lost(cli: &mqtt::AsyncClient) {
    if let Some(i_mqtt) = cli.user_data().unwrap().downcast_ref::<InnerMqtt>() {
        let mut lock_mqtt = i_mqtt.write();
        // let mut lock_mqtt = i_mqtt.write().unwrap();
        lock_mqtt.connected_to_broker = false;
        if !lock_mqtt.is_shutting_down {
            lock_mqtt.mqtt_check_task.as_mut().unwrap().start = CtrlTime::sim_time();
            log_warn!(WARN_MQTT_DISCONNECTED);
        } else {
            log_info!(INFO_MQTT_DISCONNECTED);
        }
    }
}

#[inline]
fn on_message(cli: &mqtt::AsyncClient, mqtt_msg: Option<mqtt::Message>) {
    cli.user_data().unwrap().downcast_ref::<InnerMqtt>().map(|i_mqtt| {
        mqtt_msg.map(|cli_msg| {
            let topic = cli_msg.topic();
            let payload = cli_msg.payload_str();
            match ExtMsg::new_in(Topic::from_string(topic), &payload) {
                Ok(msg) => {
                    let mut mqtt_lock = i_mqtt.write();
                    // let mut mqtt_lock = i_mqtt.write().unwrap();
                    let time = CtrlTime::sim_time();
                    if msg.topic != Topic::CLIENT_CONNECTION {
                        // aqui que trabalhamos a cache para ver se há coisas estranhas a acontecer, evitar repetir
                        // comandos, temas de idempotência e etcs
                        if mqtt_lock.msgs_cache.push(&msg.uuid) != CacheResult::Duplicate {
                            // todas as demais mensagens são passadas para o event manager.
                            mqtt_lock.msg_broker.as_ref().unwrap().reg_int_msg(MsgData::MessageIn(msg), time, DESC_MQTT_EXT_MSG_RCVD);
                        } else {
                            // msg duplicada
                            log_warn!(warn_mqtt_dup_msg(&msg.sender_id, &msg.topic.to_string()));
                        }
                        //# todas as msgs servem como client HB.
                        //# A mensagem específica de HB só se torna relevante se o cliente estiver algum tempo sem interação
                        mqtt_lock.last_heart_beat_time = time;
                    } else if let ExtMsgData::Connection(connection) = msg.data {
                        // connection messages are used only for HB-Heart Beat - not processed
                        // SPRINT CLIENT - aqui teremos que testar pelo client_id para ver quem está offline ou online.
                        // just 1 client for now - update heart beat time
                        if connection.status == CONNECTION::ONLINE.to_string() {
                            mqtt_lock.last_heart_beat_time = CtrlTime::sim_time();
                        }
                    }
                }
                Err(err) => {
                    let msg = err_unknown_mqtt_msg(&err.to_string(), topic, &payload);
                    log_warn!(&msg);
                    let mqtt_lock = i_mqtt.read();
                    // let mqtt_lock = i_mqtt.read().unwrap();
                    mqtt_lock.msg_broker.as_ref().unwrap().snd_error_to_clients(&msg, "");
                }
            }
        }) //If None, do nothing.  It is the mqtt library shutting down the consumers
    });
}
