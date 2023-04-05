//! `events.rs` is a shameless copy of `rsevents` that is an implementation of WIN32's auto-
//! and manual-reset events for the rust world.
//! Events are synchronization primitives (i.e. not implemented atop of mutexes) used to either
//! create other synchronization primitives with or for implementing signalling between threads.
//!
//! Events come in on flavor: [`CondVar`] . Internally,
//! both are implemented with the unsafe [`RawEvent`] and use the `parking_lot_core` crate to take
//! care of efficiently suspending (parking) threads while they wait for an event to become
//! signalled.
//!
//! An event is a synchronization primitive that is functionally the equivalent of an (optionally
//! gated) waitable boolean that allows for synchronization between threads. Unlike mutexes and
//! condition variables which are most often used to restrict access to a critical section, events
//! are more appropriate for efficiently signalling remote threads or waiting on a remote thread to
//! change state.
//!

use std::time::Duration;

use crate::thread_signal::raw_event::*;

/// Dimension = 24
#[derive(Debug)]
pub struct CondVar<'a> {
    pub id: &'a str,
    signal: crate::thread_signal::raw_event::RawEvent,
}

pub trait Awaitable {
    /// Check if the event has been signalled, and if not, block waiting for it to be set.
    fn wait(&self);

    /// Check if the event has been signalled, and if not, block for `limit` waiting for it to be set.
    /// Returns `true` if the event was originally set or if it was signalled within the specified
    /// duration, and `false` otherwise (if the timeout elapsed without the event becoming set).
    fn wait_for(&self, limit: Duration) -> bool;

    /// Test if an event is available without blocking, return `false` immediately if it is not
    /// set. Note that this is *not* the same as calling [`Awaitable::wait_for()`] with a `Duration` of
    /// zero, as the calling thread never yields.
    fn wait0(&self) -> bool;
}

/// A `ControllerEvent` is an event type best understood as a "waitable boolean" that efficiently
/// synchronizes thread access to a shared state, allowing one or more threads to wait for a signal
/// from one or more other threads, where the signal could have either occurred in the past or
/// could come at any time in the future.
///
/// Each time the underlying `[RawEvent]` is set [`ControllerEvent::set()`], the `ControllerEvent`
/// unparks all past waiters and allows all future waiters calling [`Awaitable::wait()`]
/// to continue without blocking (until [`ControllerEvent::reset()`] is called).
///
/// Controller events are thread-safe and may be wrapped in an [`Arc`](std::sync::Arc) to easily
/// share across threads.

impl<'a> CondVar<'a> {
    #[inline]
    pub fn new(_id: &str, state: State) -> CondVar {
        CondVar { id: _id, signal: RawEvent::new(state == State::Set) }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn is_set(&self) -> bool { self.signal.is_set() }
    /// Puts the underlying [`RawEvent`] into a set state, releasing all suspended waiters (if any)
    /// and leaving the event set for future callers.
    #[inline]
    #[rustfmt::skip]
    pub fn set(&self) { self.signal.set_all(); }

    /// Set the state of the internal event to [`State::Unset`], regardless of its current status.
    #[inline]
    #[rustfmt::skip]
    pub fn reset(&self) { self.signal.reset(); }
}

#[rustfmt::skip]
impl<'a> Awaitable for CondVar<'a> {
    /// Check if the underlying event is in a set state or wait for its state to become
    /// [`State::Set`]. The event's state is not affected by this operation, i.e. it remains set
    /// for future callers even after this function call returns.
    #[inline]
    fn wait(&self) { self.signal.unlock_all() }

    /// Check if the underlying event is in a set state (and return immediately) or wait for it to
    /// become set, up to the limit specified by the `Duration` parameter.
    ///
    /// Returns `true` if the event was initially set or if it became set within the timelimit
    /// specified. Otherwise returns `false` if the timeout elapsed without the event becoming
    /// available.
    #[inline]
    fn wait_for(&self, limit: Duration) -> bool { self.signal.wait_all_for(limit) }

    /// Test if an event is available without blocking, returning `false` immediately if it is
    /// not set.
    ///
    /// Note that this is NOT the same as calling [`Awaitable::wait_for()`] with a `Duration` of
    /// zero, as the calling thread never yields.
    #[inline]
    fn wait0(&self) -> bool { self.signal.try_unlock_all() }
}
