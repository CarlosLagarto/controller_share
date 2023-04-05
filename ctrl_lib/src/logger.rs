use flexi_logger::{Age, Cleanup, Criterion, DeferredNow, FileSpec, Level, Logger, Naming, Record, WriteMode}; //, Duplicate, detailed_format,

pub use flexi_logger::LoggerHandle;
pub use log::{debug, error, info, warn};

use crate::config::log_config::AppLog;

// isto implica que tem que existir a directoria "Data"
#[inline]
pub fn initialize_logger(app_log: &AppLog) -> LoggerHandle {
    let logger_result = Logger::try_with_str(&app_log.level)
        .unwrap()
        .use_utc()
        .log_to_file(FileSpec::default().basename(&app_log.file).directory(&app_log.directory).suppress_timestamp()) // write logs to file
        .rotate(
            // If the program runs long enough,
            Criterion::AgeOrSize(Age::Day, 268_435_456), // - create a new file every day or if it reaches  256 Mb
            Naming::Timestamps,                             // - let the rotated files have a timestamp in their name
            Cleanup::KeepLogFiles(app_log.nr_of_days_to_maintain_log_files as usize), // - keep at most X log files - default ´
        )
        // .cleanup_in_background_thread(true)// o default do flexi logger é este
        .write_mode(WriteMode::Async)
        .append() // para não truncar o log no restart do programa
        .format(custom_format)
        // .duplicate_to_stdout(Duplicate::Info) // print also to the console
        .write_mode(WriteMode::Async) // .format(detailed_format)
        .start();
    logger_result.unwrap()
}

#[inline]
fn custom_format(writer: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
    // Only write the message and the level, without the module
    let t = now.now();
    let (h, m, s, ms) = t.to_hms_milli();
    let rec = record;
    let level = rec.level();
    match level {
        Level::Error | Level::Debug | Level::Trace => {
            let mp = rec.module_path().unwrap();
            write!(writer, "{} {:02}:{:02}:{:02}.{:03} UTC {} {} {} {}", t.date(), h, m, s, ms, level, mp, rec.line().unwrap(), &rec.args())
        }
        _ => write!(writer, "{} {:02}:{:02}:{:02}.{:03} UTC {} {}", t.date(), h, m, s, ms, level, &rec.args()),
    }
}

/// Logs a message at the info level.
///
/// # Examples
///
/// let info_description = "Invalid Input";
///
/// log_info!(info_description);
///
#[macro_export]
macro_rules! log_info {
    ($x:expr) => {
        info!("{}", $x)
    };
}

/// Logs a message at the trace level.
///
/// # Examples
///
/// let trace_description = "Invalid Input";
///
/// log_trace!(trace_description);
///
#[macro_export]
macro_rules! log_trace {
    ($x:expr) => {
        trace!("{}", $x)
    };
}

/// Logs a message at the warn level.
///
/// # Examples
///
/// let warn_description = "Invalid Input";
///
/// log_warn!(warn_description);
///
#[macro_export]
macro_rules! log_warn {
    ($x:expr) => {
        warn!("{}", $x)
    };
}

/// Logs a message at the error level.
///
/// # Examples
///
/// let error_description = "Invalid Input";
///
/// log_error!(error_description);
///
#[macro_export]
macro_rules! log_error {
    ($x:expr) => {
        error!("{}", $x)
    };
}

/// Logs a message at the debug level.
///
/// # Examples
///
/// let msg_description = "Invalid Input";
///
/// log_debug!(msg_description);
///
#[macro_export]
macro_rules! log_debug {
    ($x:expr) => {
        debug!("{}", $x)
    };
}
