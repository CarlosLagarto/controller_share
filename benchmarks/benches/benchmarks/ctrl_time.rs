use alloc_counter::count_alloc;
use criterion::{black_box, BenchmarkId, Criterion};
use ctrl_lib::app_time::{ctrl_time::*, date_time::*, sunrise::*};

pub fn bench_weekday_lagarto(t: CtrlTime) {
    let _wd = t.week_day_e();
}

pub fn bench_utc_date_lagarto(t: CtrlTime) {
    let _wd = t.as_utc_date_time_e();
}

pub fn bench_from_utc_date_lagarto(t: CtrlTime) {
    let _wd = t.as_utc_date_time_e();
}

#[cfg(test)]
/// Se der negativo , é porque tm é menor que a data self
#[inline]
pub fn nr_of_days_since(base: CtrlTime, tm: CtrlTime) -> i64 {
    // use ctrl_prelude::globals::NR_NANOS_IN_A_DAY;

    ((base.0 / CtrlTime::NR_NANOS_IN_A_DAY) - (tm.0 / CtrlTime::NR_NANOS_IN_A_DAY)) as i64
}

// isto é só para bench - a remover para prd
pub fn new_calc(tm: CtrlTime) -> i64 {
    nr_of_days_since(tm, CtrlTime::from_utc_date_time_e(&DateTimeE { year: 2000, month: 1, day: 1, hour: 0, min: 0, sec: 0, nanos: 0 }))
}

pub fn bench_days_from_lagarto(t: CtrlTime) {
    let _d = new_calc(t);
}

pub fn bench_suntimes_new() {
    let elevation = 51.0;
    let latitude = 40.44072499999999;
    let longitude = -8.682944444444443;

    let (_, _) = sun_times(CtrlTime::sys_time(), latitude, longitude, elevation);
}

pub fn bench_date_time(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_date_time");

    let counts = count_alloc(|| CtrlTime::sys_time());
    println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);

    c.bench_function("bench_weekday_lagarto", |b| b.iter(|| (bench_weekday_lagarto(CtrlTime::sys_time()))));
    c.bench_function("bench_utc_date_lagarto", |b| b.iter(|| (bench_utc_date_lagarto(CtrlTime::sys_time()))));
    c.bench_function("bench_from_utc_date_lagarto", |b| b.iter(|| (bench_from_utc_date_lagarto(CtrlTime::sys_time()))));
    c.bench_function("bench_days_from_lagarto", |b| b.iter(|| (bench_days_from_lagarto(CtrlTime::sys_time()))));
    c.bench_function("bench_simulated_allocs", |b| b.iter(|| (black_box(CtrlTime::sys_time()))));

    c.finish();
}

pub fn bench_suntimes(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_suntimes");

    for i in [1, 2].iter() {
        c.bench_with_input(BenchmarkId::new("nova", i), i, |b, _| b.iter(|| bench_suntimes_new()));
    }
    c.finish();
}
