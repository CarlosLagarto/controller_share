#![allow(dead_code)]
/// SPRINT SECURITY - gather some stats to try identify DoS attempts
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, thread};

use string_concat::*;
use tiny_http::{Response, Server};

use crate::data_structs::concurrent_queue::*;
use crate::data_structs::msgs::int_message::*;
use crate::lib_serde::{Json, ConversionError};
use crate::services::electronics::actuator::{ActuatorData, ActuatorType, ActuatorCommand};
use crate::services::msg_broker::{subscriber::*, msg_brkr_svc::*};
use crate::utils::{TESTING, get_deadline_instant};
use crate::{app_time::ctrl_time::*, config::web_cfg::*};
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{log_error, log_info, logger::*, log_debug};
use ctrl_prelude::{error::*, globals::*, string_resources::*};

///  - SPRINT COMMS - enviar um sms ou uma mensagem para saber que aconteceu algo :-)
///  - por exemplo, enviar por varios canais, parametrizavel, até, com o uso, se perceber o que é mais prático.  
///     Para já de canais candidatos temos:  SMS, email, msg mqtt, twitter, whatsup.
///  - destes, o email, msg mqtt e o twitter parecem ser os únicos gratis
///  - para já usar o MQTT e o email com snmtp mail
///  - (que é o mais simples - nao requer ir perceber o oAuth da google)
///
/// Dimension = 72
#[derive(Clone)]
pub struct WebService {
    web_config: WebCfg,
    server: Arc<Server>,
    server_stop: Arc<AtomicBool>,
}

const ALIVE_REQUEST: &str = "is_alive";
const SHUTDOWN_REQUEST: &str = "shutdown";
const WEATHER_RESET_ERROR_REQUEST: &str = "weather_reset";
const WEATHER_CHANGE_SOURCE_REQUEST: &str = "weather_source_change";
const CONNECT_ID: &str ="id";

const SSL_CLIENT_S_DN: &str = "SSL_CLIENT_S_DN";

const ALIVE_RESPONSE: &str = "I was alive at:";
const SHUTDOWN_RESPONSE: &str = "Shutting down!";
const WEATHER_RESET_ERROR_RESPONSE: &str = "Weather status reseted!";
const WEATHER_CHANGE_SOURCE_RESPONSE: &str = "Weather source changed!";
const GET_WATER_HISTORY: &str = "get_water_history";
const GET_ACTUATORS_AND_SCENES: &str = "get_actuators_and_scenes";
const SET_ACTUATOR_OR_SCENE: &str = "set_actuator_or_scene";

const UNKNOWN_REQUEST_RESPONSE: &str = "Unknown command!";

const SPECIFIC_TERMINATION_ERROR: &str = "unblocked";

// production value - 500 milisecs - to be tested
// const ASYNC_WAIT_INTERVAL: u64 = 500_000_000;
// testing value - 10 minutes
const ASYNC_WAIT_INTERVAL: u64 = 600_000_000_000;

impl Default for WebService {
    #[inline]
    fn default() -> Self {
        let wc = WebCfg::new();
        // try create server.  panic if it is not possible
        let (address, port) = get_listener_ref(&wc);
        let server_base = match Server::http(string_concat!(&address, ":", port.to_string())) {
            Ok(server) => server,
            Err(e) => {
                // only one program on each port
                let msg = build_error(e.as_ref());
                eprintln!("{}", &msg);
                log_error!(&msg);
                panic!()
            }
        };
        Self {
            web_config: wc,
            server: Arc::new(server_base),
            server_stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[inline]
fn get_listener_ref(wc: &WebCfg)->(&String, &u16){
    let address : &String;
    let port : &u16;
    if !unsafe{TESTING}{
        address = &wc.address;
        port = &wc.port;
    }else{
        address = &wc.test_address;
        port = &wc.test_port;
    };
    (address, port)    
}

impl WebService {
    /// Design approach:
    /// - Launch thread A. Receives info in the "standard" channel following the global application approach
    /// - Launch thread B. Processes web server requests, synchronously - we are talking about a home systems with 1 or 2 users.
    /// - Separation between A and B, it's to isolate the web server techical details from the "business domain"
    ///
    #[inline]
    #[rustfmt::skip]
    pub fn start(&self, msg_broker: SMsgBrkr) -> thread::JoinHandle<()> {
        let web_config = self.web_config.clone();

        // 4Kb is page size in windows 32 e 64 (it seems configurable but with some pain)
        // In linux is 8Kb, configurable by $ulimits -s <value> or in /etc/sysctl.conf loaded by $sysctl -p or in /etc/security/limits.conf
        // Stacksize was found by trial an error, until stackoverflow stopped.  Thas no garanty but time will tell.
        // Started in 4 Kb and changed to 28Kb when changing byte stream to IntMessage
        // 2022/Jun/13 After reviewing data structes and reduce IntMessage fitted again in 4Kb without stack overflow
        // 2023/Fev/07 And after added watering history, increased again to 28Kb
        let builder = thread::Builder::new().name(WBSR_SERVICE_THREAD.to_owned()).stack_size(7 * STACK_SIZE_UNIT);

        let server = self.server.clone();
        let server_stop = self.server_stop.clone();

        let msg_broker = msg_broker;
        
        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                let (address, port) = get_listener_ref(&web_config);

                log_info!(string_concat!(INFO_WEB_SRVR_STARTING_1P, &address, ":", &port.to_string()));
                let mut response: Response<Cursor<Vec<u8>>>;
                let msg: String;
                let mut str_response: &str;
                let mut verb :&str;
                let mut shutting_down = false;
                let mut id_from_ssl: String = "".to_owned();
                
                loop {
                    match server.recv() {
                        Ok(req) => {
                            str_response = req.url();
                            // println!("{}", str_response);
                            // by design pathstart is always /controller_direct or /controller_direct_test
                            // coomand is the path second part
                            // and the follwing parts will be parameters, althoug I don't have a use case as off 21/Out/2022
                            let url_parts : Vec<&str> = str_response.split('/').collect();
                            verb = url_parts[3];
                            match verb {
                                ALIVE_REQUEST => response = Response::from_string(format!("{} {}", ALIVE_RESPONSE, CtrlTime::sys_time().as_rfc3339_str_e())),
                                SHUTDOWN_REQUEST => {
                                    response = Response::from_string(SHUTDOWN_RESPONSE);
                                    shutting_down = true;
                                }
                                // by design all conecting clients make this call upfront, so we can validate who is calling
                                // client authentication is two way ssl, so all clients need a certificate, unique or not, dependening on what I will decide someday,
                                // but the base idea is to have one certificate for each client, with manual certificate management, because user population will be very small.
                                CONNECT_ID => 
                                {
                                    for header in req.headers(){
                                        // println!("{}={}", header.field.as_str(), header.value.as_str());
                                        if header.field.as_str() == "SSL_CLIENT_S_DN"{
                                            //This header is injected in apache, with format SSL_CLIENT_S_DN=CN=laptop-ax-01,O=lagarto,ST=Portugal,C=PT
                                            id_from_ssl = header.value.as_str().split(&['=', ',']).collect::<Vec<&str>>()[1].to_owned();
                                            break;
                                        }
                                    }
                                    // tell the client what is their id, to use in subsequent actions
                                    response = Response::from_string(format!(r#"{{"id":"{id_from_ssl}"}}"#));
                                }
                                GET_WATER_HISTORY =>{
                                    let rsp = get_watered_cycles(&msg_broker);
                                    response = Response::from_string(rsp);
                                }
                                GET_ACTUATORS_AND_SCENES =>{
                                    let rsp = get_devices_and_scenes(&msg_broker);
                                    response = Response::from_string(rsp);
                                }
                                SET_ACTUATOR_OR_SCENE =>{
                                    // REVIEW still not tested
                                    let params_part = url_parts[4];
                                    let params: Vec<&str> = params_part.split(&['?', '=', '&']).collect();
                                    let actuator_data = ActuatorData{
                                        id : params[2].parse::<u16>().unwrap(),
                                        actuator_type: unsafe {ActuatorType::from_unchecked(params[4].parse::<u8>().unwrap())},
                                        cmd: unsafe {ActuatorCommand::from_unchecked(params[6].parse::<u8>().unwrap())},
                                        status: None,
                                    };
                                    let rsp = set_actuator_or_scene(&msg_broker, actuator_data);
                                    response = Response::from_string(rsp);
                                }
                                _ => {//ignore - but give feedback to the client
                                    log_debug!(&req.url());
                                    response = Response::from_string(UNKNOWN_REQUEST_RESPONSE);
                                } 
                            } // }//else //nop - continue listening
                            _ = req.respond(response);  //always responde something
                            if shutting_down{
                                msg_broker.snd_shut_down(); //if shutting down, broadcast that info
                            }
                        }
                        Err(e) => {
                            msg = build_error(&e);
                            if !msg.contains(SPECIFIC_TERMINATION_ERROR) { log_error!(&msg); }
                            break;
                        }
                    };
                    if server_stop.load(Ordering::Relaxed) {
                        break;
                    }
                } //end loop
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
            })
            .unwrap()
    }

    #[inline]
    pub fn terminate(&self) {
        self.server.unblock();
        self.server_stop.store(true, Ordering::Relaxed);
    }
}

// This function have several malloc and create everything from ground up, but it will be called very few times
#[inline]
fn get_watered_cycles( msg_broker: &SMsgBrkr) -> String {
    let mut rsp: String = String::from("");
    let result_tuple = ask_and_wait(msg_broker, MsgType::RspWateringHistory, MsgData::GetWateringHistory);
    let source = "resposta para o histórico da rega";
    match result_tuple{
        (Some(i_msg), _, _) =>{
            match i_msg.data {
                MsgData::RspWateringHistory(water_history) => {
                    let result = water_history.json();
                    handle_result(result, &mut rsp);
                }
                _ =>{
                    handle_error(source, &mut rsp); 
                }
            }
        }
        (None, is_time_out, _) => {
            handle_timeout(is_time_out, &mut rsp, "resposta para o histórico da rega");
        },
    }
    msg_broker.unregister_and_unsubscribe(Subscriber::Web);
    rsp
}

// This function have several malloc and create everything from ground up, but it will be called very few times
#[inline]
fn get_devices_and_scenes( msg_broker: &SMsgBrkr) -> String {
    let mut rsp: String = String::from("");
    let result_tuple = ask_and_wait(msg_broker, MsgType::RspDevicesAndScenes, MsgData::GetDevicessAndScenes);
    let source = "resposta para os devices e cenários";
    match result_tuple{
        (Some(i_msg), _, _) =>{
            match i_msg.data {
                MsgData::RspDevicesAndScenes(actuator_data) => {
                    let result = actuator_data.json();
                    handle_result(result, &mut rsp);
                }
                _ =>{
                    handle_error(source, &mut rsp); 
                }
            }
        }
        (None, is_time_out, _) => {
            handle_timeout(is_time_out, &mut rsp, "resposta para os devices e cenários");
        },
    }
    msg_broker.unregister_and_unsubscribe(Subscriber::Web);
    rsp
}

// This function have several malloc and create everything from ground up, but it will be called very few times
fn set_actuator_or_scene( msg_broker: &SMsgBrkr, actuator_data: ActuatorData) -> String {
    let mut rsp: String = String::from("");
    let result_tuple = ask_and_wait(msg_broker, MsgType::RspActuatorOrScene, MsgData::SetActuatorOrScene(actuator_data));
    let source = "resposta para o setting de um devices ou cenário";
    match result_tuple{
        (Some(i_msg), _, _) =>{
            match i_msg.data {
                MsgData::RspDevicesAndScenes(actuator_data) => {
                    let result = actuator_data.json();
                    handle_result(result, &mut rsp);
                }
                _ =>{
                    handle_error(source, &mut rsp); 
                }
            }
        }
        (None, is_time_out, _) => {
            handle_timeout(is_time_out, &mut rsp, source);
        },
    }
    msg_broker.unregister_and_unsubscribe(Subscriber::Web);
    rsp
}

#[inline]
fn handle_timeout(is_time_out: bool, rsp: &mut String, source: &str) {
    if is_time_out{
        warn!("{}: timeout", source);
        *rsp = String::from("timeout: na origem");

    }else{
        error!("{}: mensagem de resposta não prevista", source);
        *rsp = String::from("erro: mensagem de resposta não prevista");
    }
}

#[inline]
fn ask_and_wait(msg_broker: &Arc<MsgBrkr>, msg_type: MsgType, msg_data: MsgData) -> (Option<IntMessage>, bool, bool) {
    let web_subs_queue = Arc::new(MtDeque::new());
    // register in broker
    msg_broker.register_in_broker(Subscriber::Web, web_subs_queue.clone());
    msg_broker.subscribe(msg_type, Subscriber::Web);
    // pass the request to the broker, to be caught by other subscriber
    msg_broker.reg_int_msg(msg_data, CtrlTime::sys_time());
    // and we will wait for an answer, with timeout...just in case
    // wait for msg_brk response -     //wait 500 ms
    web_subs_queue.recv_timeout(get_deadline_instant(ASYNC_WAIT_INTERVAL))
}

#[inline]
fn handle_result(result: Result<String, ConversionError>, rsp: &mut String) {
    match result{
        Ok(str_result) =>{
            *rsp = str_result; 
        },
        Err(e)=>{
            log_error!(e.to_string());
            *rsp = String::from("erro: mensagem de resposta inválida na origem."); 
        }
    }
}

#[inline]
fn handle_error(source: &str, rsp: &mut String) {
    error!("{}: mensagem de resposta não prevista.", source);
    *rsp = String::from("erro: mensagem de resposta não prevista");
}
