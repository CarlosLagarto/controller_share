use criterion::{black_box, Criterion};
// use ctrl_lib::config::log_config::AppLog;
// use ctrl_lib::{logger, log_info};
// use ctrl_prelude::string_resources::INFO_LAST_WORDS;
// use flexi_logger::{Logger, FileSpec, Criterion as log_criterion, Naming, Cleanup, WriteMode, Age, DeferredNow, Record, Level};
use syslog::{BasicLogger, Facility, Formatter3164, Logger, LoggerBackend};

use syslog;

use log::{info, LevelFilter};

// #[inline]
// fn custom_format(writer: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
//     // Only write the message and the level, without the module
//     let t = now.now();
//     let (h, m, s, ms) = t.to_hms_milli();
//     let rec = record;
//     let level = rec.level();
//     match level {
//         Level::Error | Level::Debug | Level::Trace => {
//             let mp = rec.module_path().unwrap();
//             write!(writer, "{} {:02}:{:02}:{:02}.{:03} UTC {} {} {} {}", t.date(), h, m, s, ms, level, mp, rec.line().unwrap(), &rec.args())
//         }
//         _ => write!(writer, "{} {:02}:{:02}:{:02}.{:03} UTC {} {}", t.date(), h, m, s, ms, level, &rec.args()),
//     }
// }

// pub fn bench_log_existing_flexi_logger() {
//     // let logger_handle = logger::initialize_logger(AppLog::new());

//     let _logger_result = Logger::try_with_str("info")
//     .unwrap()
//     .use_utc()
//     .log_to_file(FileSpec::default().basename("log_test.log").directory("/home/lagarto/DEV/RUST/controller").suppress_timestamp()) // write logs to file
//     .rotate(
//         // If the program runs long enough,
//         log_criterion::AgeOrSize(Age::Day, 268_435_456), // - create a new file every day or if it reaches  256 Mb
//         Naming::Timestamps,                             // - let the rotated files have a timestamp in their name
//         Cleanup::KeepLogFiles(10), // - keep at most X log files - default ´
//     )
//     // .cleanup_in_background_thread(true)// o default do flexi logger é este
//     .write_mode(WriteMode::Async)
//     .append() // para não truncar o log no restart do programa
//     .format(custom_format)
//     // .duplicate_to_stdout(Duplicate::Info) // print also to the console
//     .write_mode(WriteMode::Async) // .format(detailed_format)
//     .start();
//     // println!("{}", logger_result);
//     // let _logger_handle = logger_result.unwrap();

//     log_info!(INFO_LAST_WORDS);
// }

pub fn bench_log_without_log_sup(logger: &mut Logger<LoggerBackend, Formatter3164>) {
    // let formatter = Formatter3164 { facility: Facility::LOG_USER, hostname: None, process: "controller".into(), pid: 42 };

    // match syslog::unix(formatter) {
    //     Err(e) => println!("impossible to connect to syslog: {:?}", e),
    //     Ok(mut writer) => {
    //         writer.err("hello world").expect("could not write error message");
    //     }
    // }
    logger.err("hello world").expect("could not write error message");
}

pub fn bench_log_with_log_sup() {
    // let formatter = Formatter3164 { facility: Facility::LOG_USER, hostname: None, process: "controller".into(), pid: 0 };

    // let logger = syslog::unix(formatter).expect("could not connect to syslog");
    // _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger))).map(|()| log::set_max_level(LevelFilter::Info));

    info!("hello world");
}

pub fn bench_logss(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_logs");

    let formatter = Formatter3164 { facility: Facility::LOG_USER, hostname: None, process: "controller".into(), pid: 0 };
    let formatter1 = Formatter3164 { facility: Facility::LOG_USER, hostname: None, process: "controller".into(), pid: 0 };
    let logger1 = syslog::unix(formatter).expect("could not connect to syslog");
    let mut logger = syslog::unix(formatter1).expect("could not connect to syslog");
    _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger1))).map(|()| log::set_max_level(LevelFilter::Info));
    
    // c.bench_function("bench_log_existing_flexi_logger", |b| b.iter(|| black_box(bench_log_existing_flexi_logger())));
    c.bench_function("bench_log_without_log_sup", |b| b.iter(|| black_box(bench_log_without_log_sup(&mut logger))));
    c.bench_function("bench_log_with_log_sup", |b| b.iter(|| black_box(bench_log_with_log_sup())));
    c.finish();
}
