
use std::time::{Duration, Instant};

use ctrl_lib::config::watering_config::WateringConfig;

#[test]
fn test_file_open() {
    let mut t1: Instant = Instant::now();
    let mut t2: Instant;
    let mut d: Duration = t1 - t1;

    let n = 10;
    for _i in 0..n {
        t1 = Instant::now();
        let obj = WateringConfig::new();
        t2 = Instant::now();
        println!("{}", elapsed_mcs(t1, t2, "abre ficheiro watering toml:  "));
        drop(obj);
        d = d + (t2 - t1);
    }
    println!("{}", elapsed_base_n(d, n, "tempo médio abertura fiheiro watering toml: "));
}

#[test]
fn test_file_save() {
    let mut t1: Instant = Instant::now();
    let mut t2: Instant;
    let mut d: Duration = t1 - t1;
    let obj = WateringConfig::new();

    let n = 10;
    for _i in 0..n {
        t1 = Instant::now();
        let _res = obj.save_if_updated();
        t2 = Instant::now();
        println!("{}", elapsed_mcs(t1, t2, "grava ficheiro watering toml: "));
        d = d + (t2 - t1);
    }
    println!("{}", elapsed_base_n(d, n, "tempo médio gravar ficheiro watering toml: "));
}

fn elapsed_base(t: std::time::Duration, msg: &str) -> String {
    format!("{} Time (μ secs): {:.3}", msg, t.as_micros() as f64 + t.subsec_nanos() as f64 * 1e-3)
}
pub fn elapsed_mcs(t1: Instant, t2: Instant, msg: &str) -> String {
    let t = t2 - t1;
    elapsed_base(t, msg)
}

fn elapsed_base_n(t: std::time::Duration, n: u64, msg: &str) -> String {
    format!("{} Total Time (μ secs): {:.3}", msg, (t.as_micros() as f64 + t.subsec_nanos() as f64 * 1e-3) / n as f64)
}
