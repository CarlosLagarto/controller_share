#![allow(dead_code)]
/// SPRINT SECURITY - pensar aqui em estatisticas para identificar nr de chamadas (DoS) acima do razoável que pode indicar mais alguém a aceder á máquina
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use ctrl_prelude::error::build_error;
use string_concat::*;
use tiny_http::{Response, Server};

use crate::config::web_cfg::*;
use crate::services::msg_broker::svc::*;
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{log_error, log_info, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};

/// Este web web_server é o plano B para dar comandos ao controlador, com curl a dar comandos GET ou com um simples browser.
/// A estratégia geral é de ter sempre duas formas de fazer as coisas para aumentar a robustez do sistema.
///
///  - SPRINT COMMS - enviar um sms ou uma mensagem para saber que aconteceu algo :-)
///  - por exemplo, enviar por varios canais, parametrizavel, até, com o uso, se perceber o que é mais prático.  
///     Para já de canais candidatos temos:  SMS, email, msg mqtt, twitter, whatsup.
///  - destes, o email, msg mqtt e o twitter parecem ser os únicos gratis
///  - para já usar o MQTT e o email com snmtp mail
///  - (que é o mais simples - nao requer ir perceber o oAuth da google)
///
/// Dimensão = 48
#[derive(Clone)]
pub struct WebService {
    web_config: WebCfg,
    server: Arc<Server>,
    server_stop: Arc<AtomicBool>,
}

const ALIVE_REQUEST: &str = "/is_alive";
const SHUTDOWN_REQUEST: &str = "/shutdown";
const WEATHER_RESET_ERROR_REQUEST: &str = "/weather_reset";
const WEATHER_CHANGE_SOURCE_REQUEST: &str = "/weather_source_change";


const ALIVE_RESPONSE: &str = "I am alive!";
const SHUTDOWN_RESPONSE: &str = "Shutting down!";
const WEATHER_RESET_ERROR_RESPONSE: &str = "Weather status reseted!";
const WEATHER_CHANGE_SOURCE_RESPONSE: &str = "Weather source changed!";

const UNKNOWN_REQUEST_RESPONSE: &str = "Unknown command!";

const SPECIFIC_TERMINATION_ERROR: &str = "unblocked";

impl Default for WebService {
    #[inline]
    fn default() -> Self {
        let wc = WebCfg::new();
        //tentamos criar o servidor
        // NOTE - se não tivermos web web_server mas tivermos MQTT ainda conseguimos controlar isto...
        // ...pensar como se pode mudar e ir testando entre um e outro....
        // Para já não é tema, a não ser quando quisermos robustecer mais isto
        let server_base = match Server::http(string_concat!(&wc.address, ":", wc.port.to_string())) {
            Ok(server) => server,
            Err(e) => {
                //apenas pode haver um servidor web neste port, por inerência da função
                log_error!(build_error(e.as_ref()));
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

        //4Kb é aparentemente o valor da pagina no windows 32b e 64b
        //eventualmente compilação condicional para o caso do linux que ainda vou ter que ver validar
        // este valor foi por tentativa e erro, até deixar de ter stackoverflow nesta thread.
        // e isto estava em 4 Kb e mudou quando revi as mensagens internas de uma stream de bytes, para o IntMessage
        // 2022/Jun/13 Bem, e depois de rever as estruturas de dados e reduzir o tamanho do IntMessage, voltou a caber em 4Kb sem stack overflow
        let builder = thread::Builder::new().name(WBSR_SERVICE_THREAD.to_owned()).stack_size(STACK_SIZE_UNIT);

        let server = self.server.clone();
        let server_stop = self.server_stop.clone();

        let msg_broker = msg_broker;
        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                log_info!(string_concat!(INFO_WEB_SRVR_STARTING_1P, &web_config.address, ":", web_config.port.to_string()));
                loop {
                    match server.recv() {
                        Ok(req) => {
                            match req.url() {
                                ALIVE_REQUEST => {
                                    let response = Response::from_string(ALIVE_RESPONSE);
                                    let _ = req.respond(response);
                                }
                                SHUTDOWN_REQUEST => {
                                    let response = Response::from_string(SHUTDOWN_RESPONSE);
                                    let _ = req.respond(response);
                                    //e agora damos indicação que foi pedido o shutdown
                                    msg_broker.snd_shut_down(DESC_SHUTDOWN_CMD_FROM_WEB);
                                }
                                _ => {
                                    let response = Response::from_string(UNKNOWN_REQUEST_RESPONSE);
                                    let _ = req.respond(response);
                                } //ignoramos - mas damos uma mensagem de erro/feedback
                            } // }//else //não á nada a processar - continuamos á escuta
                        }
                        Err(e) => {
                            let msg = build_error(&e);
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
