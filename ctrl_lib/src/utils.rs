use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fmt, fs::File, io::Read};

use lexical::{ToLexical, BUFFER_SIZE};
use parking_lot::RwLock;

use crate::{app_time::ctrl_time::*, string_concat::*};
use ctrl_prelude::string_resources::*;

pub type ArcRw<T> = Arc<RwLock<T>>;

#[cfg(debug_assertions)]
pub static mut THREAD_COUNT: usize = 1;

pub static mut TESTING: bool = false;

pub fn arc_rw<T>(x: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(x))
}

#[inline]
pub fn build_abs_file_path(file: &str) -> String {
    let exe = std::env::current_exe().unwrap();
    let file = exe.parent().unwrap_or_else(|| panic!("{}", err_wrong_file_path(file))).join(file);
    file.as_os_str().to_str().unwrap().to_owned()
}

#[inline]
pub fn deserialize_file<T>(file: &str) -> T
where
    T: for<'de> serde::Deserialize<'de>,
{
    let sfile: &str = &build_abs_file_path(file);
    let file_result = File::open(sfile);
    let mut buffer: Vec<u8> = Vec::with_capacity(256); //o maior ficheiro que tenho é de 192 bytes, pelo que um buffer de 256 bytes chegará para o meu use case
    if let Ok(mut file) = file_result {
        file.read_to_end(&mut buffer).unwrap_or_else(|e| panic!("{}", err_read_cfg_file(&e.to_string())));
        toml::from_slice::<T>(&buffer).unwrap_or_else(|e| panic!("{}", err_deser_cfg_file(&e.to_string())))
    } else {
        panic!("{}", err_read_cfg_file(&format!("{sfile}{file_result:?}")))
    }
}

// After benchmarking this implementation seems to be the more efficiently stable
#[inline]
pub fn elapsed_dyn(duration_nanos: u64) -> String {
    let mut output: String = String::with_capacity(40);
    let mut f = |p, v| fmt::write(&mut output, format_args!("Total Time {p}{v:>.3}")).unwrap();
    if duration_nanos < 1000 {
        f("(ns): ", duration_nanos as f64);
    } else {
        let nano: f64 = duration_nanos as f64;
        let mic: f64 = nano * 0.001;

        if duration_nanos < 1000000 {
            f("(us): ", mic);
        } else {
            let mil: f64 = mic * 0.001;
            if duration_nanos < 1000000000 {
                f("(ms): ", mil);
            } else {
                let sec: f64 = mil * 0.001;
                if duration_nanos < 60000000000 {
                    f("(s): ", sec);
                } else {
                    let min: f64 = sec * 0.0166666666666667;
                    if duration_nanos < 3600000000000 {
                        f("(min.): ", min);
                    } else {
                        let hrs: f64 = min * 0.0166666666666667;
                        if duration_nanos < 86400000000000 {
                            f("(hours): ", hrs);
                        } else {
                            let day: f64 = hrs * 0.0416666666666667;
                            f("(days): ", day);
                        }
                    }
                }
            }
        }
    }
    output
}

#[inline]
pub fn conv_int<T>(val: T) -> String
where
    T: ToLexical,
{
    let mut buffer = [b'\x00'; BUFFER_SIZE];
    let buf: &mut [u8];
    unsafe {
        buf = val.to_lexical_unchecked(&mut buffer);
    }
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

#[macro_export]
macro_rules! ifmt {
    ($a:expr) => {
        conv_int($a)
    };
}

#[inline]
pub fn get_deadline_duration(interval_secs_in_nano: u64) -> Duration {
    let receiver_deadline_duration_nanos = CtrlTime::sys_time_duration().subsec_nanos() as u64;
    //seconds round up considering the interval_secs_in_nano param
    Duration::from_nanos(interval_secs_in_nano - receiver_deadline_duration_nanos.rem_euclid(interval_secs_in_nano))
}

/// Returns the next deadline moment
#[inline]
pub fn get_deadline_instant(interval_in_nanos: u64) -> Instant {
    let receiver_deadline_duration_nanos = CtrlTime::sys_time_duration().subsec_nanos() as u64;
    //seconds round up considering the interval_secs_in_nano param
    Instant::now() + Duration::from_nanos(interval_in_nanos - receiver_deadline_duration_nanos)
}

/// initialize backup, tipically with one second
/// retry in 1 second, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65136 ...
pub struct Backoff {
    pub should_backoff: bool,
    /// in seconds
    pub minimum_backoff_time: u64,
    /// in seconds
    pub maximum_backoff_time: u64,
    pub backoff_time: u64,
    pub start_time: u64,

}

impl Backoff {
    #[inline]
    pub fn new(minimum_backoff_time: u64, max_backoff_time: Option<u64>) -> Self {
        let maximum_backoff_time: u64;
        if let Some(max) = max_backoff_time {
            maximum_backoff_time = max;
        } else {
            maximum_backoff_time = CtrlTime::MAX;
        }
        Self {
            should_backoff: false,
            minimum_backoff_time,
            maximum_backoff_time,
            backoff_time: minimum_backoff_time,
            start_time: 0,
        }
    }

    #[inline]
    pub fn start(&mut self) {
        self.should_backoff = true;
        self.start_time = CtrlTime::sys_time().0;
    }

    #[inline]
    pub fn stop(&mut self) {
        self.backoff_time = self.minimum_backoff_time;
        self.should_backoff = false;
    }

    #[inline]
    // Returns next deadline Instant or None
    // updates deadline for the next Instant
    pub fn set_next_deadline(&mut self) {
        if self.should_backoff && self.minimum_backoff_time <= self.maximum_backoff_time {
            self.minimum_backoff_time += self.minimum_backoff_time ^ 2;
        }
    }

    #[inline]
    pub fn is_time(&self, time: CtrlTime) -> bool {
        self.should_backoff && self.start_time + self.minimum_backoff_time >= time.0
    }
}

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
}

impl StringUtils for str {
    #[inline]
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            }
            else { break; }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            }
            else { break; }
        }
        &self[byte_start..byte_end]
    }
}
