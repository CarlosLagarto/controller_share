// use libc::tm;

use std::mem::MaybeUninit;

use ctrl_prelude::globals::*;
use crate::app_time::ctrl_time::CtrlTime;

const FRAC_1_60: f64 = 0.0166666666666667;

#[inline]
#[rustfmt::skip]
pub fn min_to_nano(minutes: f32) -> u64 { (minutes as f64 * 60. * GIGA_F) as u64 }

#[inline]
#[rustfmt::skip]
pub fn min_to_sec_f32(minutes: f32) -> f32 { minutes * 60. }

#[inline]
#[rustfmt::skip]
pub fn sec_to_min(seconds: f32) -> f32 { (seconds as f64 * FRAC_1_60) as f32 }

#[inline]
#[rustfmt::skip]
pub fn nano_to_min(nanos: u64) -> f32 { (nanos as f64 * FRAC_1_60 * NANO_F) as f32 }

pub struct UtcOffset {
    // #[allow(clippy::missing_docs_in_private_items)]
    pub hours: i8,
    // #[allow(clippy::missing_docs_in_private_items)]
    pub minutes: i8,
    // #[allow(clippy::missing_docs_in_private_items)]
    pub seconds: i8,
}

impl UtcOffset {
    pub const fn __from_hms_unchecked(hours: i8, minutes: i8, seconds: i8) -> Self {
        if hours < 0 {
            debug_assert!(minutes <= 0);
            debug_assert!(seconds <= 0);
        } else if hours > 0 {
            debug_assert!(minutes >= 0);
            debug_assert!(seconds >= 0);
        }
        if minutes < 0 {
            debug_assert!(seconds <= 0);
        } else if minutes > 0 {
            debug_assert!(seconds >= 0);
        }
        debug_assert!(hours.unsigned_abs() < 24);
        debug_assert!(minutes.unsigned_abs() < 60);
        debug_assert!(seconds.unsigned_abs() < 60);

        Self {
            hours,
            minutes,
            seconds,
        }
    }

    pub const UTC: Self = Self::__from_hms_unchecked(0, 0, 0);

    pub const fn from_whole_seconds(seconds: i32) -> Self {
        // ensure_value_in_range!(seconds in -86_399 => 86_399);
        debug_assert!(seconds > -83_399);
        debug_assert!(seconds < 86_399);
        Self::__from_hms_unchecked((seconds / 3_600) as _, ((seconds / 60) % 60) as _, (seconds % 60) as _)
    }
}

#[cfg(any(
    target_os = "redox",
    target_os = "linux",
    target_os = "l4re",
    target_os = "android",
    target_os = "emscripten",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "haiku",
))]

fn tm_to_offset(tm: libc::tm) -> Option<UtcOffset> {
    let seconds = tm.tm_gmtoff.try_into().unwrap(); //.ok()?;
    Some(UtcOffset::from_whole_seconds(seconds))
}

/// This method will remain `unsafe` until `std::env::set_var` is deprecated or has its behavior
/// altered. This method is, on its own, safe. It is the presence of a safe, unsound way to set
/// environment variables that makes it unsafe.
unsafe fn timestamp_to_tm(timestamp: i64) -> Option<libc::tm> {
    extern "C" {
        #[cfg_attr(target_os = "netbsd", link_name = "__tzset50")]
        fn tzset();
    }

    // The exact type of `timestamp` beforehand can vary, so this conversion is necessary.
    #[allow(clippy::useless_conversion)]
    let timestamp = timestamp.try_into().ok()?;

    let mut tm = MaybeUninit::uninit();

    // Update timezone information from system. `localtime_r` does not do this for us.
    //
    // Safety: tzset is thread-safe.
    unsafe { tzset() };

    // Safety: We are calling a system API, which mutates the `tm` variable. If a null
    // pointer is returned, an error occurred.
    let tm_ptr = unsafe { libc::localtime_r(&timestamp, tm.as_mut_ptr()) };

    if tm_ptr.is_null() {
        None
    } else {
        // Safety: The value was initialized, as we no longer have a null pointer.
        Some(unsafe { tm.assume_init() })
    }
}

pub fn get_utc_offset() -> Option<UtcOffset> {
    let tm = unsafe { timestamp_to_tm(CtrlTime::sys_time().ux_ts() as i64) };
    tm_to_offset(tm.unwrap())
}
