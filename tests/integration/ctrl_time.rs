use std::time::Instant;
use alloc_counter::count_alloc;
use num_enum::TryFromPrimitive;

use ctrl_lib::{
    app_time::{ctrl_time::*, date_time::*},
    utils::elapsed_dyn,
};

#[test]
fn ctrl_time_eq() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(0);
    assert!(t1 == t2, "left: {}, right:{}", t1, t2);
}

#[test]
#[should_panic]
fn ctrl_time_fail_eq() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(1);
    assert!(t1 == t2, "left: {}, right:{}", t1, t2);
}

#[test]
fn ctrl_time_lt() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(1);
    assert!(t1 < t2);
}

#[test]
#[should_panic]
fn ctrl_time_fail_lt() {
    let t1 = CtrlTime(2);
    let t2 = CtrlTime(1);
    assert!(t1 < t2, "left: {}, right:{}", t1, t2);
}
#[test]
fn ctrl_time_le() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(1);
    assert!(t1 <= t2, "left: {}, right:{}", t1, t2);
}

#[test]
fn ctrl_time_gt() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(1);
    assert!(t2 > t1, "left: {}, right:{}", t1, t2);
}

#[test]
fn ctrl_time_ge() {
    let t1 = CtrlTime(0);
    let t2 = CtrlTime(1);

    assert!(t2 >= t1, "left: {}, right:{}", t1, t2);
}

#[test]
fn test_from_date_time() {
    let d = DateTimeE::new_ymdhms(2022, 4, 1, 0, 0, 0);

    let t = CtrlTime::from_utc_date_time_e(&d);
    let d1 = t.as_utc_date_time_e();
    // println!("time: {}", t);
    // println!("d: {:?}", d);
    // println!("d1: {:?}", d1);

    assert!(d == d1);
}

#[test]
fn test_ux_ts() {
    let d = DateTimeE::new_ymdhms(2022, 4, 1, 0, 0, 0);

    let t = CtrlTime::from_utc_date_time_e(&d);

    assert!(t.ux_ts() == 1_648_771_200);
}

#[test]
fn test_ts_hr() {
    let d = DateTimeE::new_ymdhms(2022, 4, 1, 0, 0, 0);

    let t = CtrlTime::from_utc_date_time_e(&d);

    assert!(t.ts() == 1_648_771_200.);
}

#[test]
fn test_nr_days_since() {
    let d1 = DateTimeE::new_ymdhms(2022, 3, 1, 1, 0, 0);
    let d2 = DateTimeE::new_ymdhms(2022, 3, 3, 1, 0, 0);

    let t1 = CtrlTime::from_utc_date_time_e(&d1);
    let t2 = CtrlTime::from_utc_date_time_e(&d2);

    assert!(t2.nr_of_days_since(t1) == 2);
}

#[test]
fn test_nr_days_since_perf_variants() {
    let d1 = DateTimeE::new_ymdhms(2022, 3, 1, 1, 0, 0);
    let d2 = DateTimeE::new_ymdhms(2022, 3, 3, 1, 0, 0);

    let t1 = CtrlTime::from_utc_date_time_e(&d1);
    let t2 = CtrlTime::from_utc_date_time_e(&d2);

    let counts = count_alloc(|| t2.nr_of_days_since(t1));
    println!("Allocations: {}  Reallocations: {}  Deallocations: {}", counts.0 .0, counts.0 .1, counts.0 .2);
    assert!(t2.nr_of_days_since(t1) == 2);

    const N: u64 = 10000;
    let mut tm_total: u64 = 0;
    for _i in 0..N {
        let tm2 = Instant::now();
        t2.nr_of_days_since(t1);
        tm_total += tm2.elapsed().as_nanos() as u64;
    }
    // println!("tempp médio vb ns: {:0.0}", (tm_total / N as f64) * GIGA_F);
    println!("{}", elapsed_dyn((tm_total / N) as u64));
}

#[allow(non_camel_case_types)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// "dicionário" para ajudar no cálculo dos specific days!
/// Su|Mo|Tu|We|Th|Fr|St
///
///:                           | `Su` | `Mo` | `Tu` | `We` | `Th` | `Fr` | `St` |
///--------------------------- | ---- | ---- | ---- | ---- | ---- | ---- | ---- |
///:                           |   0  |  1   |   2  |   3  |   4  |   5  |  6   |
///
pub enum WEEK_DAYS {
    Su = 0,
    Mo = 1,
    Tu = 2,
    We = 3,
    Th = 4,
    Fr = 5,
    St = 6,
}

// #[cfg(test)]
pub fn next_week_day_test(ref_week_day: WEEK_DAYS, wd: WEEK_DAYS) -> u8 {
    let a = ref_week_day as i8;
    let b = wd as i8;
    let c: i8 = b - a;
    if b >= a {
        c as u8
    } else {
        (CtrlTime::WEEK_DAYS_I8 + c) as u8
    }
}

// #[cfg(test)]
// #[inline]
pub fn week_day_difference(first: u8, next: u8) -> u8 {
    let mut x = first as i8;
    x -= next as i8;
    if (0..CtrlTime::WEEK_DAYS_I8).contains(&x) {
        x as u8
    } else {
        (x + CtrlTime::WEEK_DAYS_I8) as u8
    }
}

#[test]
fn test_next_week_day_test() {
    let first = WEEK_DAYS::Su;
    let nexts: &[WEEK_DAYS] = &[WEEK_DAYS::Su, WEEK_DAYS::Mo, WEEK_DAYS::Tu, WEEK_DAYS::We, WEEK_DAYS::Th, WEEK_DAYS::Fr, WEEK_DAYS::St];

    let arra_next_week_day: Vec<u8> = nexts.iter().map(|i| next_week_day_test(first.clone(), (*i).clone())).collect();
    let arra_diff: Vec<u8> = nexts.iter().map(|i| week_day_difference(first.clone() as u8, (*i).clone() as u8)).collect();

    assert_eq!(arra_next_week_day, [0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(arra_diff, [0, 6, 5, 4, 3, 2, 1]);
}

#[test]
fn test_is_leap() {
    assert!(is_leap(2021) == 0);
}

#[test]
fn test_is_not_leap() {
    assert!(is_leap(2020) == 1);
}

#[test]
fn test_all_leap_years_in_the_universe() {
    let list: [u16; 25] = [
        1972, 1976, 1980, 1984, 1988, 1992, 1996, 2000, 2004, 2008, 2012, 2016, 2020, 2024, 2028, 2032, 2036, 2040, 2044, 2048, 2052, 2056, 2060,
        2064, 2068,
    ];
    for year in 1970..2071 {
        if list.contains(&year) {
            assert!(is_leap(year)==1);
        }else{
            assert!(is_leap(year)==0);
        }
    }
}

#[test]
fn test_min(){
    let date1 = CtrlTime::from_utc_parts(2023, 1,  1 , 0, 0, 0);
    let date2 = CtrlTime::from_utc_parts(2023, 1,  1 , 1, 0, 0);

    assert!(date1.min(date2) == date1);
}

#[test]
fn test_max(){
    let date1 = CtrlTime::from_utc_parts(2023, 1,  1 , 0, 0, 0);
    let date2 = CtrlTime::from_utc_parts(2023, 1,  1 , 1, 0, 0);

    assert!(date1.max(date2) == date2);
}

#[test]
fn test_clamp(){
    let date1 = CtrlTime::from_utc_parts(2023, 1,  1 , 0, 0, 0);
    let date2 = CtrlTime::from_utc_parts(2023, 1,  1 , 1, 0, 0);

    let date3 = CtrlTime::from_utc_parts(2023, 1,  1 , 2, 0, 0);
    assert!(date3.clamp(date1, date2) == date2);
}

#[test]
fn test_utc_as_date_time_e(){
    let date1 = CtrlTime::from_utc_parts(2023, 1,  1 , 0, 0, 0);
    let date2 = DateTimeE::new_ymdhmsn(2023, 1, 1, 0, 0, 0, 0);
    assert!(date1.as_utc_date_time_e() == date2);
}