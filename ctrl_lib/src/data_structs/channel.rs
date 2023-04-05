// DESING COMMENTS 
// tenho um bounded channel aqui dimensionado para 5 mensagens no canal.
// deve chegar , mas eventualmente carece de teste a ver se este numero introduz algum constrangimento nos tempos de processamento
// uma vez que chegando a este numero, o sender bloqueia á espera que algum receiver leia a mensagem.
// ver comenários no msg broker para a racional do cap do channel
pub use crossbeam_channel::{bounded, Receiver, RecvError, RecvTimeoutError, Select, SendError, Sender};

use super::msgs::int_message::*;

pub type EventReceiver = Receiver<IntMessage>;
pub type EventSender = Sender<IntMessage>;

pub type ReceiverError = RecvError;
pub type ChannelSendError<T> = SendError<T>;
pub type ReceiveTimeoutError = RecvTimeoutError;

const BOUNDED_CAP :usize = 5;
// #[derive(Debug)]
/// Dimensão = 32 bytes
pub struct Channel {
    pub rx: EventReceiver, 
    pub tx: EventSender,  
}

#[inline]
#[rustfmt::skip]
pub fn channel() -> (EventSender, EventReceiver) { bounded(BOUNDED_CAP) }

impl Default for Channel {
    #[inline]
    fn default() -> Self {
        let (tx, rx): (EventSender, EventReceiver) = bounded(BOUNDED_CAP);//ver comentários no msg broker para a racional do cap do channel
        Self { rx, tx }
    }
}

impl Channel {
    #[inline]
    #[rustfmt::skip]
    pub const fn build(rx: EventReceiver, tx: EventSender) -> Channel { Channel { rx, tx } }
}

impl Clone for Channel {
    #[inline]
    #[rustfmt::skip]
    fn clone(&self) -> Channel { Channel { rx: self.rx.clone(), tx: self.tx.clone() } }
}

