
use assert_float_eq::*;

use ctrl_lib::app_time::date_time::*;
use ctrl_lib::app_time::sunrise::*;
use ctrl_lib::app_time::ctrl_time::*;
use ctrl_prelude::globals::*;

const DIFF: f64 = 60. * 6. * GIGA_F; // 5 MINUTOS

#[test]
fn test_suntime_one() {
    let elevation = 51.0;
    let latitude = 40.44072499999999;
    let longitude = -8.682944444444443;

    let mut oracle: Vec<[CtrlTime; 3]> = Vec::with_capacity(154);
    load_oracle_mar_2022(&mut oracle);

    let (rise, set) = sun_times(oracle[0][0], latitude, longitude, elevation);
    println!("rise {}", &rise.as_rfc3339_str_e());
    println!("set {}", &set.as_rfc3339_str_e());

    println!("diff rise: {}", (oracle[0][1].0 as f64 - rise.0 as f64).abs() / GIGA_F   );
    println!("diff set: {}", (oracle[0][2].0 as f64 - set.0 as f64).abs() / GIGA_F   );
    //set
    assert!(
        expect_float_absolute_eq!(oracle[0][2].0 as f64, set.0 as f64, DIFF).is_ok(),
        "set diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
        oracle[0][2].as_rfc3339_str_e(),
        set.as_rfc3339_str_e(),
        (oracle[0][2].0 as f64 - set.0 as f64).abs() / GIGA_F
    );

    //rise
    assert!(
        expect_float_absolute_eq!(oracle[0][1].0 as f64, rise.0 as f64, DIFF).is_ok(),
        "rise diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
        oracle[0][1].as_rfc3339_str_e(),
        rise.as_rfc3339_str_e(),
        (oracle[0][1].0 as f64 - rise.0 as f64).abs() / GIGA_F //para dar segundos
    );

}

#[test]
fn test_suntime_oct_2022() {
    let elevation = 51.0;
    let latitude = 40.44072499999999;
    let longitude = -8.682944444444443;

    let mut oracle: Vec<[CtrlTime; 3]> = Vec::with_capacity(154);
    load_oracle_oct_2022(&mut oracle);

    let mut diff: f64 = 0.;
    for ar in oracle {
        let (rise, set) = sun_times(ar[0], latitude, longitude, elevation);

        assert!(
            expect_float_absolute_eq!(ar[1].0 as f64, rise.0 as f64, DIFF).is_ok(),
            "rise diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
            ar[1].as_rfc3339_str_e(),
            rise.as_rfc3339_str_e(),
            (ar[1].0 as f64 - rise.0 as f64).abs(),
        );

        diff = diff.max((ar[1].0 as f64 - rise.0 as f64).abs());

        assert!(
            expect_float_absolute_eq!(ar[2].0 as f64, set.0 as f64, DIFF).is_ok(),
            "set diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
            ar[2].as_rfc3339_str_e(),
            set.as_rfc3339_str_e(),
            (ar[2].0 as f64 - set.0 as f64).abs()
        );

        diff = diff.max((ar[2].0 as f64 - set.0 as f64).abs());
    }

    println!("Maxima diferença neste set: {}", diff);
}

#[test]
fn test_suntime_mar_2022() {
    let elevation = 51.0;
    let latitude = 40.44072499999999;
    let longitude = -8.682944444444443;

    let mut oracle: Vec<[CtrlTime; 3]> = Vec::with_capacity(154);
    load_oracle_mar_2022(&mut oracle);

    let mut diff: f64 = 0.;
    for ar in oracle {
        let (rise, set) = sun_times(ar[0], latitude, longitude, elevation);

        assert!(
            expect_float_absolute_eq!(ar[1].0 as f64, rise.0 as f64, DIFF).is_ok(),
            "rise diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
            ar[1].as_rfc3339_str_e(),
            rise.as_rfc3339_str_e(),
            (ar[1].0 as f64 - rise.0 as f64).abs(),
        );
        diff = diff.max((ar[1].0 as f64 - rise.0 as f64).abs());

        assert!(
            expect_float_absolute_eq!(ar[2].0 as f64, set.0 as f64, DIFF).is_ok(),
            "set diff fora limites.  Oracle; {}  Calc: {}  Diff:{}",
            ar[2].as_rfc3339_str_e(),
            set.as_rfc3339_str_e(),
            (ar[2].0 as f64 - set.0 as f64).abs()
        );
        diff = diff.max((ar[2].0 as f64 - set.0 as f64).abs());
    }
    println!("Maxima diferença neste set: {}", diff);
}

fn load_oracle_mar_2022(oracle: &mut Vec<[CtrlTime; 3]>) {
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 1, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 1, 7, 9,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 1, 18, 26,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 2, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 2, 7, 7,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 2, 18, 27,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 3, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 3, 7, 6,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 3, 18, 28,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 4, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 4, 7, 4,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 4, 18, 29,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 5, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 5, 7, 2,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 5, 18, 31,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 6, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 6, 7, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 6, 18, 32,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 7, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 7, 6, 59,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 7, 18, 33,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 8, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 8, 6, 58,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 8, 18, 34,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 9, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 9, 6, 56,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 9, 18, 35,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 10, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 10, 6, 55,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 10, 18, 36,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 11, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 11, 6, 53,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 11, 18, 37,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 12, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 12, 6, 51,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 12, 18, 38,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 13, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 13, 6, 50,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 13, 18, 39,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 14, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 14, 6, 48,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 14, 18, 40,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 15, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 15, 6, 47,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 15, 18, 41,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 16, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 16, 6, 45,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 16, 18, 42,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 17, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 17, 6, 43,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 17, 18, 43,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 18, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 18, 6, 42,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 18, 18, 45,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 19, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 19, 6, 40,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 19, 18, 46,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 20, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 20, 6, 38,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 20, 18, 47,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 21, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 21, 6, 37,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 21, 18, 48,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 22, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 22, 6, 35,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 22, 18, 49,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 23, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 23, 6, 33,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 23, 18, 50,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 24, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 24, 6, 32,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 24, 18, 51,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 25, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 25, 6, 30,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 25, 18, 52,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 26, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 26, 6, 28,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 26, 18, 53,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 27, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 27, 6, 27,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 27, 18, 54,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 28, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 28, 6, 25,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 28, 18, 55,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 29, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 29, 6, 24,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 29, 18, 56,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 30, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 30, 6, 22,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 30, 18, 57,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 31, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 31, 6, 20,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 3, 31, 18, 58,0)),
    ]);
}

fn load_oracle_oct_2022(oracle: &mut Vec<[CtrlTime; 3]>) {
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 1, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 1, 6, 31,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 1, 18, 17,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 2, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 2, 6, 32,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 2, 18, 15,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 3, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 3, 6, 33,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 3, 18, 14,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 4, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 4, 6, 34,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 4, 18, 12,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 5, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 5, 6, 35,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 5, 18, 11,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 6, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 6, 6, 36,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 6, 18, 9,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 7, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 7, 6, 37,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 7, 18, 7,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 8, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 8, 6, 38,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 8, 18, 6,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 9, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 9, 6, 39,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 9, 18, 4,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 10, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 10, 6, 40,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 10, 18, 3,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 11, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 11, 6, 41,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 11, 18, 1,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 12, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 12, 6, 42,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 12, 17, 59,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 13, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 13, 6, 43,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 13, 17, 58,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 14, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 14, 6, 44,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 14, 17, 56,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 15, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 15, 6, 45,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 15, 17, 55,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 16, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 16, 6, 47,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 16, 17, 53,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 17, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 17, 6, 48,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 17, 17, 52,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 18, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 18, 6, 49,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 18, 17, 50,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 19, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 19, 6, 50,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 19, 17, 49,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 20, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 20, 6, 51,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 20, 17, 48,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 21, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 21, 6, 52,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 21, 17, 46,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 22, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 22, 6, 53,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 22, 17, 45,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 23, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 23, 6, 54,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 23, 17, 43,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 24, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 24, 6, 55,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 24, 17, 42,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 25, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 25, 6, 56,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 25, 17, 41,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 26, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 26, 6, 58,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 26, 17, 39,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 27, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 27, 6, 59,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 27, 17, 38,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 28, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 28, 7, 00,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 28, 17, 37,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 29, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 29, 7, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 29, 17, 35,0)),
    ]);
    oracle.push([
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 30, 0, 1,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 30, 7, 3,0)),
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(2022, 10, 30, 17, 34,0)),
    ]);
}
