use std::time::Duration;

use lazy_static::*;

use crate::thread_signal::cond_var::{Awaitable, CondVar};
use crate::thread_signal::raw_event::State;
use crate::utils::{arc_rw, ArcRw};
use crate::{log_warn, logger::*};
use ctrl_prelude::string_resources::*;

// delay nanos seconds - 5 secs  - DB mnt is 5 secs after
pub const NEW_DAY_START_DELAY: u64 = 5_000_000_000; 
pub const DB_MAINT_START_DELAY: u64 = 10_000_000_000; 
pub const WTR_DAILY_MNT_DELAY: u64 = 15_000_000_000; 

// timming is coordinated - although saying that the wtr daily starts 5'' after bd mnt, and having a 10' timeout de 10' in bd mnt id stupid :-)
const EXPECTED_MAX_DB_MNT_DURATION: u64 = 600; // db mnt doesn't take 10'....have to make load test
const EXPECTED_MAX_NEW_DAY_DURATION: u64 = 5; // db mnt doesn't take 5''....have to make load test

const NEW_DAY_DURATION: Duration = Duration::new(EXPECTED_MAX_NEW_DAY_DURATION, 0);
const DB_MNT_DURATION: Duration = Duration::new(EXPECTED_MAX_DB_MNT_DURATION, 0);

#[inline]
pub fn new_day_and_db_mnt_sync() {
    wait_for_new_day_task();
    wait_for_db_maint_task();
}

#[inline]
pub fn wait_for_new_day_task() {
    if !NEW_DAY_SIG.read().wait_for(NEW_DAY_DURATION) {
        log_warn!(WARN_NEW_DAY_PROCESS_DELAY);
    };
}

#[inline]
pub fn wait_for_db_maint_task() {
    if !DB_MAINT_SIG.read().wait_for(DB_MNT_DURATION) {
        log_warn!(WARN_BD_MAINT_PROCESS_DELAY);
    };
}

lazy_static! {
    pub static ref DB_MAINT_SIG: ArcRw<CondVar<'static>> = arc_rw(CondVar::new("DBMaintSignal", State::Set));
}
lazy_static! {
    pub static ref NEW_DAY_SIG: ArcRw<CondVar<'static>> = arc_rw(CondVar::new("NewDaySignal", State::Set));
}
