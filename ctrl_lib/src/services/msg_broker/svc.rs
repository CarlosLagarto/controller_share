//! Broker de mensagens
//!
//! A ideia do design é ter um canal por tipo de mensagem (EVENT_TYPE)
//!
//! Os subscritores podem subscrever qualquer um dos eventos definidos
//!
//! Cada evento disponibiliza um canal para comunicação
//!
//! É da responsabilidade dos subscritores ficarem á escuta do canal subscrito
//!
//! Entretanto (2022/Maio/31) apanhei um tema interessante que deriva do design/implementação escolhido
//! Inicialmente fui pela solução simples de channel unbounded
//! Naturalmente isso para um sistema embebido não é interessante porque não estava verdadeiramente a controlar a quantidade de memória usada
//! Neste raciocinio mudei recentemente para channel bounded (e arbitrariamente escolhi a capacidade 5)
//! Nesta mudança coloquei em evidencia uma limitação do design, que é a de partilhar apenas um canal de send para n subscritores
//! O que quer dizer que havendo varias mensagens num curto espaço de tempo, e estando a mainthread ocupada a fazer coisas, estava a criar uma
//! dependencia e lock porque não estava a despachar (ler) as mensagens em menos tempo do que o tempo de as criar .
//!
//! 1ª tentativa para o cap de mensagens, mantendo este design - colocar um channel de send para cada tipo de mensagem, apesar de estarem disponiveis no codigo,
//! isso implica aumentar a contenção no send, porque os channels estão dentro do inner strut o que implica mais um read()
//! Portanto mantendo este racional, o cap escolhido deverá numa primeira abordagem = cap * nr de tipos de mensagens
//!
//! O resultado desta primeira tentativa mostrou que há um outro tema qualquer algures.
//! Há um momento em que algo acontece e as mensagens não são processadas, acumulando msgs e a partir daí independentemente do cap do channel,
//! a coisa irá sempre rebentar a memória.  
//! It turns out que estava a criar dois subscritores para o mesmo tipo de mensagem, na mesma thread,
//! e por isso lia uma das mensagens mas a outra ficava ali pendurada - DESIGN FLAW
//!
//!
//! Centraliza a informação de todos os eventos que circulam na aplicação
//!
//! Captura os eventos e regista-os na queue, e distribui/dispatch para quem os subscreveu
//!
//! Desta forma o log de eventos também fica centralizado em vez de espalhado pela aplicação, espero...
//!
//! O Manager está numa thread própria, para que as threads que publicam retornem imediatamente, e esta thread
//! acorda e faz as cenas, sem "pendurar" as threads que chamam:
//!  - mqtt (blocking nos sockets)
//!  - web - esta está pendente de resposta web
//!  - io (blocking on io externo)
//!  - sensores - no futuro
//!  - comandos, que ou vem da web, ou do mqtt, ou do sistema operativo(shutdown, etc.) ou o ctrl c/interrupt do teclado
//!  - erros naquilo que não se perceber :-)
//!

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use arrayvec::ArrayVec;
use ctrl_prelude::error::build_error;
use parking_lot::RwLock;
// pub use no_deadlocks::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_hash::FxHashMap;

use crate::app_time::ctrl_time::CtrlTime;
use crate::controller_sync::new_day_and_db_mnt_sync;
use crate::data_structs::msgs::{ext_message::ExtMsg, int_message::*, log_error::*};
use crate::data_structs::{channel::*, timer_signals::*};
#[cfg(debug_assertions)]
use crate::log_debug;
use crate::services::msg_broker::{errors::*, subscriber::*};
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{log_error, log_info, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};

type BrokerResult = std::result::Result<(), ChannelSendError<IntMessage>>;
type MsgSubscribedList = ArrayVec<Subscriber, MAX_SUBSCRIBERS>;

///Dimensão = 8
pub type SMsgBrkr = Arc<MsgBrkr>;

/// Dimensão = 48
pub struct MsgBrkr {
    pub inner: Arc<RwLock<InnerBroker>>,
    pub evt_in: Channel,
    pub working: Arc<AtomicBool>,
}

impl MsgBrkr {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(InnerBroker::default()));
        MsgBrkr {
            inner,
            evt_in: Channel::default(),
            working: Arc::new(AtomicBool::new(false)),
        }
    }

    #[inline]
    pub fn register_in_broker(&self, subscriber: Subscriber) -> Channel {
        let (tx, rx) = channel();
        let channel = Channel::build(rx.clone(), tx);
        let o_channel: Option<Channel>;
        {
            // tenho aqui um clone, mas estamos a falr de um u8
            o_channel = self.inner.write().subscribers.insert(subscriber.clone(), channel);
            // o_channel = self.inner.write().unwrap().subscribers.insert(subscriber_name.to_owned(), channel);
        };
        if let Some(channel) = o_channel {
            // já estava registada esta chave
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_dup_subs(subscriber.to_string()));
            channel
        } else {
            //para avaliar os numeros a reservar para a capacidade.
            #[cfg(debug_assertions)]
            log_info!(dbg_brkr_nr_of_subs(self.inner.read().subscribers.len()));
            // log_info!(dbg_brkr_nr_of_subs(self.inner.read().unwrap().subscribers.len()));
            Channel::build(rx, self.evt_in.tx.clone())
        }
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
        //para avaliar os numeros a reservar para a capacidade.
        #[cfg(debug_assertions)]
        log_info!(dbg_brkr_nr_of_msgs_subs(&msg_type.to_string(), self.inner.read().subscribed_msgs[index].len()));
        // log_info!(dbg_brkr_nr_of_msgs_subs(&msg_type.to_string(), self.inner.read().unwrap().subscribed_msgs[index].len()));
        result
    }

    #[inline]
    #[rustfmt::skip]
    pub fn get_msg_brkr_handle(&self) -> EventSender { self.evt_in.tx.clone() }

    #[inline]
    pub fn reg_int_msg(&self, msg_data: MsgData, time: CtrlTime, description: &str) {
        //para avaliar os números a reservar para a capacidade.
        #[cfg(debug_assertions)]
        log_info!(dbg_brkr_nr_of_msg_in(self.evt_in.tx.len()));
        // apanhei durante os testes que quando ninguém subscreve as mensagens, a capacidade do chanel enche....pelo que se pensa: porquê enviar msgs qd não há subscribers.
        let index = msg_data.tipo();
        let have_subscriber: bool;
        {
            let read_guard = self.inner.read();
            // let read_guard = inner_broker.read().unwrap(); // usto está aqui para se for preciso despistar deadlocks
            let subscritores = &read_guard.subscribed_msgs[index];
            have_subscriber = !subscritores.is_empty();
        }
        if have_subscriber {
            let send_result = self.evt_in.tx.send(IntMessage::build(msg_data, time, description));
            self.handle_send_msg_result(send_result);
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn snd_error_to_clients(&self, err_msg: &str, descricao: &str) {
        self.reg_int_msg(
            MsgData::ClientError(LogError { error: err_msg.to_string(), }),
            CtrlTime::sim_time(),
            descricao,
        );
    }

    #[inline]
    pub fn snd_shut_down(&self, descricao: &str) {
        self.reg_int_msg(MsgData::ShutDown(TimerSignal::Shutdown), CtrlTime::sim_time(), descricao);
    }

    #[inline]
    pub fn snd_terminate(&self) {
        self.handle_send_msg_result(self.evt_in.tx.send(IntMessage::build(MsgData::StopMessageBroker, CtrlTime::sim_time(), INFO_BRKR_TERMINATE_MSG)))
    }

    #[inline]
    pub fn snd_status_changed(&self, descricao: &str, time: CtrlTime) {
        self.reg_int_msg(MsgData::StateChanged, time, descricao);
    }

    #[inline]
    pub fn snd_ext_msg(&self, message: ExtMsg) {
        self.reg_int_msg(MsgData::MessageOut(message), CtrlTime::sim_time(), BRKR_STR_EXT_MSG);
    }

    #[inline]
    fn handle_send_msg_result(&self, result: BrokerResult) {
        if let Err(e) = result {
            log_error!(BrokerError::IssueSendingMsg(e.to_string()));
        };
    }

    #[inline]
    #[rustfmt::skip]
    pub fn start(&self) -> thread::JoinHandle<()> {
        let builder = thread::Builder::new().name(EVBR_SERVICE_THREAD.to_owned()).stack_size(18 * STACK_SIZE_UNIT);
        let inner_broker = self.inner.clone();
        let local_working = self.working.clone();
        let broker_rx = self.evt_in.rx.clone();

        let handler = builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                log_info!(INFO_BRKR_THREAD_START);
                local_working.store(true, Ordering::SeqCst);
                while let Ok(msg) = broker_rx.recv() {
                    new_day_and_db_mnt_sync();
                    match msg.data {
                        MsgData::StopMessageBroker => local_working.store(false, Ordering::SeqCst),
                        _ => process_msg(Box::new(msg), &inner_broker),
                    }
                    if !local_working.load(Ordering::SeqCst) {
                        break;
                    };
                } //fim do loop
                  // err termina o while loop Receiving on a Disconnected channel - may happen durante os testes e os asserts de validação que panicam, por definição
                log_info!(INFO_BRKR_THREAD_STOP);
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
            })
            .unwrap();
        self.working.store(true, Ordering::SeqCst);
        handler
    }

    #[inline]
    #[rustfmt::skip]
    pub fn working(&self) -> bool { self.working.load(Ordering::SeqCst) }
}

// extrai esta função do loop para poder medir a performance e as alocações
#[inline]
pub fn process_msg(msg: Box<IntMessage>, inner_broker: &Arc<RwLock<InnerBroker>>) {
    let index = msg.data.tipo();
    let no_subscriber_string: String = msg.data.to_string();

    log_info!(info_broker_gen(&no_subscriber_string, &msg.descricao));

    let read_guard = inner_broker.read();
    // let read_guard = inner_broker.read().unwrap();
    let subscritores = &read_guard.subscribed_msgs[index];

    // para todos os subscritores desta mensagem, vamos enviá-la
    for subs in subscritores.iter() {
        let channel = read_guard.subscribers.get(subs).unwrap();
        // #[cfg(debug_assertions)]
        // {
        //     match msg.data {
        //         MsgData::MessageOut(_) => println!("message out"),
        //         MsgData::StopMQTT => println!("mqtt stop"),
        //         _ => println!("outra"),
        //     }
        // }
        channel.tx.send(*msg.clone()).unwrap_or_else(|err| log_error!(err_snd_brkr_msg(subs.to_string(), &build_error(&err))));
    }
    #[cfg(debug_assertions)]
    // Recebeu-se uma msg sem subscritor.  isto é para debug para ver a utilidade das subscrições e se não for usada, eliminar para prd
    if subscritores.is_empty() {
        log_debug!(dbg_brkr_no_sbs(&no_subscriber_string, &msg.descricao));
    }
}

/// Dimensão = 216
pub struct InnerBroker {
    // Na revisão para mais eficiência (sim, não se melhora a eficiência sem medir, mas isto é um "learning projet")
    // Inicialmente tinhamos um Vec<HashSet>
    // Depois mudei para [HashSet]
    // E depois pensei que o HashSet tinha implicito a decisão de resolver a colisão de nomes no módulo.
    // E neste ultimo ponto pensei que não faz sentido este módulo preocupar-se com isso.
    // A decisão de resolução de duplicados é uma coisa que alguém se deve ou pode preocupar, mas fora do módulo
    // E finalmente decidi que o nr de subscritores é por volta dos 20, mas a dinãmica da coisa faz com que não faça sentido ter um
    // array estaticamente dimensionado, porque não sei nesta fase a cardinalidade das subscrições
    // Pelo que estabilizei para já num vec, que é dinamico, mas não nos preocupamos com nomes duplicados.  Se os houver, que se resolva fora.
    // E entretanto 2022/06/01 voltei ao hashmap, porque tinha uma complicação desnecessária no código e um problema de design
    // onde tinha channels por tipo de mensagem, o que era uma parvoice.
    // O que é preciso é channels por subscriber.  Não sei onde tinha a cabeça ou qual o racional que me levou para ali.
    // desta forma fica mais clean
    // e em tese vai simplificar o tema de não ter que ter um select para todos os tipos de mensagem, mas apenas 1 por subscriber.
    pub subscribers: FxHashMap<Subscriber, Channel>,
    pub subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE],
}

impl Default for InnerBroker {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self {
        let subscribers: FxHashMap<Subscriber, Channel> = FxHashMap::default();
        let subscribed_msgs: [MsgSubscribedList; COUNT_EVENT_TYPE] = Default::default();
        Self { subscribers, subscribed_msgs, }
    }
}
