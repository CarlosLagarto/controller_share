use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use parking_lot_core as plc;
use parking_lot_core::ParkResult;

/// Dimension = 1
#[derive(Debug)]
pub struct RawEvent(AtomicBool); // true for set, false for unset

/// A representation of the state of an event, which can either be `Set` (i.e. signalled,
/// ready) or `Unset` (i.e. not ready).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// The event is available and call(s) to [`Awaitable::wait()`] will go through without
    /// blocking, i.e. the event is signalled.
    Set,
    /// The event is unavailable and calls to [`Awaitable::wait()`] will block until the event
    /// becomes set, i.e. the event is unsignalled.
    Unset,
}

impl RawEvent {
    #[inline]
    pub fn new(state: bool) -> RawEvent {
        let event = RawEvent(AtomicBool::new(false));
        event.0.store(state, Ordering::Relaxed);
        event
    }
#[inline]
#[rustfmt::skip]
pub fn is_set(&self) -> bool { self.0.load(Ordering::Relaxed) }

    #[inline]
    /// # Safety
    pub unsafe fn suspend_all(&self) {
        plc::park(self as *const RawEvent as usize, || !self.try_unlock_all(), || {}, |_, _| {}, plc::DEFAULT_PARK_TOKEN, None);
    }

    #[inline]
    /// Attempts to obtain the event (without locking out future callers). Returns true upon success.
    #[rustfmt::skip]
    pub fn try_unlock_all(&self) -> bool { self.0.load(Ordering::Acquire) }

    #[inline]
    /// Trigger the event, releasing all waiters
    pub fn set_all(&self) {
        self.0.store(true, Ordering::Release);
        unsafe { plc::unpark_all(self as *const RawEvent as usize, plc::DEFAULT_UNPARK_TOKEN) };
    }

    #[inline]
    #[rustfmt::skip]
    pub fn unlock_all(&self) {
        if !self.try_unlock_all() { unsafe { self.suspend_all(); } }
    }

    #[inline]
    /// Put the event in a locked (reset) state.
    #[rustfmt::skip]
    pub fn reset(&self) { self.0.store(false, Ordering::Release); }

    #[inline]
    pub fn wait_all_for(&self, limit: Duration) -> bool {
        let end = Instant::now() + limit;
        let wait_result =
            unsafe { plc::park(self as *const RawEvent as usize, || !self.try_unlock_all(), || {}, |_, _| {}, plc::DEFAULT_PARK_TOKEN, Some(end)) };

        wait_result != ParkResult::TimedOut
    }
}
