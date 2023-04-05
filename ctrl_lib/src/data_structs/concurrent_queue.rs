// Depois de partir muita pedra com o crossbeam channel, como estrutura de eleição para trocar dados entre threads, esbarrei num bug que não consegui resolver.
// O canal teimava em "pendurar" porque  o reader não dava despacho ao writer.
// Acabei por implementar isto, e ver-me livre de mais de 30K de codigo da library, porque no essencial, só preciso mesmo disto
// Acabou por ser a solução que já tinha no Python
// Deve haver qualquer coisa no crossbeam, mas não quis perder mis tempo a fazer debug a codigo dos outros.  Basta ter que fazer ao meu :-)
use parking_lot::{Condvar, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::data_structs::msgs::int_message::*;
use ctrl_prelude::globals::SHUTTING_DOWN;

pub const MAX_ELEMENTS: usize = 50;

/// Dimension 64 qd o tipo é IntMessage
pub struct MtDeque<T: Clone> {
    deque: Mutex<VecDeque<T>>,
    cv_empty: Condvar,
    cv_full: Condvar,
    len: AtomicU8,
}

pub type SMtDeque = Arc<MtDeque<IntMessage>>;

unsafe impl<T: Clone> Sync for MtDeque<T> {}
unsafe impl<T: Clone> Send for MtDeque<T> {}

impl<T: Clone> MtDeque<T> {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::with_capacity(MAX_ELEMENTS)),
            len: AtomicU8::new(0),
            cv_empty: Condvar::new(),
            cv_full: Condvar::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed) as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.load(Ordering::Relaxed) == 0
    }

    /// Importante - os signals dependem de duas coisas - a condição da queue (não vazia ou cheia)
    /// E do SHUTTING_DOWN = true 
    /// Sem o indicador de shutting down, o signal não dispara, e a queue não acorda.
    #[inline]
    pub fn terminate(&self) {
        self.cv_empty.notify_all();
        self.cv_full.notify_all();
    }

    /// Devolve 1 parametro <br>
    /// Se foi shutdown: bool = false <br>
    #[inline]
    pub fn send(&self, elem: T) -> bool {
        let mut guard = self.deque.lock();
        self.cv_full.wait_while(&mut guard, closure_snd);
        if !unsafe { SHUTTING_DOWN } {
            guard.push_back(elem);
            self.len.fetch_add(1, Ordering::Relaxed);
            self.cv_empty.notify_all();
            true
        } else {
            false
        }
    }

    /// Devolve o elemento ou None se foi shutdown>
    #[inline]
    pub fn recv(&self) -> Option<T> {
        let mut guard = self.deque.lock();
        self.cv_empty.wait_while(&mut guard, closure_rcv);

        if !unsafe { SHUTTING_DOWN } {
            let popped_elem = guard.pop_front().unwrap();
            self.len.fetch_sub(1, Ordering::Relaxed);
            self.cv_full.notify_all();
            Some(popped_elem)
        } else {
            None
        }
    }

    /// Devolve 3 parametros <br>
    /// O elemento: Option<T> , ou None caso algum das flags seguintes seja true <br>
    /// Se foi time out : bool <br>
    /// Se foi shutdown: bool <br>
    #[inline]
    pub fn recv_timeout(&self, interval: Instant) -> (Option<T>, bool, bool) {
        let mut guard = self.deque.lock();
        let timeout_result = self.cv_empty.wait_while_until(&mut guard, closure_rcv, interval);
        if timeout_result.timed_out() {
            (None, true, false)
        } else if !unsafe { SHUTTING_DOWN } {
            let popped_elem = guard.pop_front().unwrap();
            self.len.fetch_sub(1, Ordering::Relaxed);
            self.cv_full.notify_all();
            (Some(popped_elem), false, false)
        } else {
            (None, false, true)
        }
    }
}

#[inline]
fn closure_snd<T>(vars: &mut VecDeque<T>) -> bool {
    vars.len() >= MAX_ELEMENTS && !unsafe { SHUTTING_DOWN }
}

#[inline]
fn closure_rcv<T>(vars: &mut VecDeque<T>) -> bool {
    vars.is_empty() && !unsafe { SHUTTING_DOWN }
}
