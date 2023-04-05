use criterion::{BenchmarkId, Criterion};
use ctrl_lib::app_time::{
    ctrl_time::CtrlTime,
    schedule::Schedule,
    schedule_params::{ScheduleRepeat, ScheduleRepeatUnit, ScheduleStop},
};

pub fn bench_schedule1(sched: &mut Schedule) {
    let _res = sched.set_next_event();
}

pub fn bench_schedule2(sched: &mut Schedule) {
    let _res = sched.set_next_event();
}

pub fn bench_schedule_daily_1_day(sched: &mut Schedule) {
    let _res = sched.get_next_event();
}

pub fn bench_schedule_every_day_10_day(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

pub fn bench_schedule_weekly_1_week(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

pub fn bench_schedule_hourly(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

pub fn bench_schedule_every_hour(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

pub fn bench_schedule_every_minutes(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

pub fn bench_schedule_every_seconds(sched: &mut Schedule) {
    let _schedule = sched.get_next_event();
}

// pub fn bench_schedule_never_never(sched: &mut Schedule) {
//     let _schedule = sched.get_next_event();
// }

pub fn bench_schedule_specific_wd(sched: &Schedule) {
    let _schedule = sched.get_next_event();
}

// isto será para saber mais objetivamente o peso da criação da struct
pub fn bench_schedule_base_struct_creation() {
    let _sched14: Schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime(0),
        repeat_spec_wd: 0b01111111, //String::from("Su|Mo|Tu|We|Th|Fr|St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
}

pub fn bench_schedule(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_schedule");
    c.bench_function("bench_schedule_base_struct_creation", |b| b.iter(|| bench_schedule_base_struct_creation()));
    let sched = &mut Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime(0),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    c.bench_function("bench_schedule1", |b| b.iter(|| bench_schedule1(sched)));

    sched.start = CtrlTime(1000);
    c.bench_function("bench_schedule2", |b| b.iter(|| bench_schedule2(sched)));
    sched.start = CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0);
    sched.repeat_kind = ScheduleRepeat::Every;
    sched.repeat_every_unit = ScheduleRepeatUnit::Days;
    c.bench_function("bench_schedule_daily_1_day", |b| b.iter(|| bench_schedule_daily_1_day(sched)));

    sched.repeat_every_qty = 10;
    sched.repeat_every_unit = ScheduleRepeatUnit::Days;
    c.bench_function("bench_schedule_every_day_10_day", |b| b.iter(|| bench_schedule_every_day_10_day(sched)));

    sched.repeat_every_qty = 1;
    sched.repeat_every_unit = ScheduleRepeatUnit::Weeks;
    c.bench_function("bench_schedule_weekly_1_week", |b| b.iter(|| bench_schedule_weekly_1_week(sched)));

    sched.repeat_every_qty = 1;
    sched.repeat_every_unit = ScheduleRepeatUnit::Hours;
    c.bench_function("bench_schedule_hourly", |b| b.iter(|| bench_schedule_hourly(sched)));

    sched.repeat_kind = ScheduleRepeat::Every;
    sched.repeat_every_unit = ScheduleRepeatUnit::Hours;
    c.bench_function("bench_schedule_every_hour", |b| b.iter(|| bench_schedule_every_hour(sched)));

    sched.repeat_kind = ScheduleRepeat::Every;
    sched.repeat_every_unit = ScheduleRepeatUnit::Minutes;
    c.bench_function("bench_schedule_every_minutes", |b| b.iter(|| bench_schedule_every_minutes(sched)));

    sched.repeat_every_unit = ScheduleRepeatUnit::Seconds;
    c.bench_function("bench_schedule_every_seconds", |b| b.iter(|| bench_schedule_every_seconds(sched)));

    // sched.start = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    // sched.repeat_kind = ScheduleRepeat::Never;
    // sched.repeat_every_unit = ScheduleRepeatUnit::Seconds;
    // c.bench_function("bench_schedule_never_never", |b| b.iter(|| bench_schedule_never_never(sched)));

    sched.start = CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0);
    sched.repeat_kind = ScheduleRepeat::SpecificWeekday;
    sched.repeat_every_qty = 2;
    sched.repeat_spec_wd = 0b01000000; //String::from("Su");
    sched.repeat_every_unit = ScheduleRepeatUnit::Seconds;
    c.bench_function("bench_schedule_specific_wd_1_day", |b| b.iter(|| bench_schedule_specific_wd(sched)));

    c.finish();
}

pub fn bench_schedule_specific_wd_group(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_schedule specific wd");

    let sched1 = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime(0),
        repeat_spec_wd: 0b01000000, //String::from("Su"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let mut sched2: Schedule = sched1.clone();
    sched2.repeat_spec_wd = 0b01100000; //String::from("Su|Mo");

    let mut sched3: Schedule = sched1.clone();
    sched3.repeat_spec_wd = 0b01110000; //String::from("Su|Mo|Tu");

    let mut sched4: Schedule = sched1.clone();
    sched4.repeat_spec_wd = 0b01111000; //String::from("Su|Mo|Tu|We");

    let mut sched5: Schedule = sched1.clone();
    sched5.repeat_spec_wd = 0b01111100; //String::from("Su|Mo|Tu|We|Th");

    let mut sched6: Schedule = sched1.clone();
    sched6.repeat_spec_wd = 0b01111110; //String::from("Su|Mo|Tu|We|Th|Fr");

    let mut sched7: Schedule = sched1.clone();
    sched7.repeat_spec_wd = 0b01111111; //String::from("Su|Mo|Tu|We|Th|Fr|St");

    for i in [sched1, sched2, sched3, sched4, sched5, sched6, sched7].iter() {
        c.bench_with_input(BenchmarkId::new("days selected", i.repeat_spec_wd.clone()), i, |b, _| b.iter(|| bench_schedule_specific_wd(i)));
    }

    c.finish();
}
