use ctrl_lib::app_time::{ctrl_time::*, schedule::*, schedule_params::*};

use ctrl_prelude::globals::GIGA_U;

#[test]
fn test_sched_run_daily_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        // println!("calc: {} expected: {}", new_time.as_rfc3339_str_e(), CtrlTime::from_utc_parts(2022, 3, 10, 7, 5, 0).as_rfc3339_str_e());
        //estamos a testar com after = start pelo que o start é válido
        assert_eq!(new_time.0, CtrlTime::from_utc_parts(2022, 3, 10, 7, 5, 0).0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_weekly_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Weeks,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 15, 8, 5, 0), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 16, 7, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_hourly_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Hours,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 9, 8, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_weeks_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Weeks,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 15, 8, 5, 0), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 16, 7, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_days1_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 10, 7, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_days10_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 10,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 19, 7, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_3day_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 3,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 15, 8, 5, 0), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 18, 7, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_hours_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Hours,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 9, 8, 5, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_minutes1_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Minutes,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 9, 7, 6, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_minutes115_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 7, 5, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 115,
        repeat_every_unit: ScheduleRepeatUnit::Minutes,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    let expected = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        // println!("calc: {} expected: {}", new_time.as_rfc3339_str_e(), expected.as_rfc3339_str_e());

        assert!(new_time == expected, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_seconds_never_stop() {
    let schedule = Schedule {
        start: CtrlTime(1000 * GIGA_U),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(schedule.start, &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime(1001 * GIGA_U), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_every_2_second_never_stop() {
    let schedule = Schedule {
        start: CtrlTime(GIGA_U),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime(GIGA_U), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime(3 * GIGA_U), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_never_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime(1000), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_su_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b01000000, //String::from("Su"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 12, 9, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 13, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff: i64 = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_su_mo_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 7, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: WeekDaysBC::Sunday as u8 | WeekDaysBC::Monday as u8, //String::from("Su|Mo"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 12, 10, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 13, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(new_time, target));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_tu_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00010000, //String::from("Tu"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 12, 10, 0, 0), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 15, 9, 0, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_we_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00001000, //String::from("We"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);

    if let Some(new_time) = _res.unwrap() {
        assert!(new_time == CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0), "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_th_never_stop_before_first() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000100, //String::from("Th"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]

fn test_sched_run_specific_wd_th_never_stop_after_second() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 10, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000100, //String::from("Th"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 10, 10, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 17, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_wd_fr_never_stop_before_first_day() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00001010, //String::from("We|Fr"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_3_days_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: WeekDaysBC::Sunday as u8 | WeekDaysBC::Monday as u8 | WeekDaysBC::Tuesday as u8,
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 13, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_4_days_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b01111000, //String::from("Su|Mo|Tu|We"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    // a data do start_ts é terça ás 9 horas
    // corre ás seg, ter, qua, e qui
    // o after_ts quarta , mas antes da hora de execução na propria quarta
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    // quer dizer que o resultado esperado é quarta
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_5_days_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b01111100, //String::from("Su|Mo|Tu|We|Th"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_6_days_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b01111110, //String::from("Su|Mo|Tu|We|Th|Fr"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_7_days_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 8, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b01111111, //String::from("Su|Mo|Tu|We|Th|Fr|St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 8, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}
#[test]
fn test_sched_run_specific_wd_fr_never_stop_after() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 4, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000010, //String::from("Fr"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };

    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 11, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 11, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_st_never_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000001, //String::from("St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    // executou no ultimo sab dia 5
    // a executar após qua ás 10
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 9, 10, 0, 0), &schedule);
    // o sab seguinte a dia 5 é dia 12
    let target = CtrlTime::from_utc_parts(2022, 3, 12, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_st_never_stop_one_month_after() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000001, //String::from("St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    // executou no ultimo sab dia 5
    // a executar após qua ás 10
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 4, 2, 10, 0, 0), &schedule);
    // o sab seguinte a dia 5 é dia 12
    let target = CtrlTime::from_utc_parts(2022, 4, 9, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_st_never_stop_one_year_after() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000001, //String::from("St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    // executou no ultimo sab dia 5
    // a executar após qua ás 10
    let _res = find_next_event(CtrlTime::from_utc_parts(2023, 5, 27, 10, 0, 0), &schedule);
    // o sab seguinte a dia 5 é dia 12
    let target = CtrlTime::from_utc_parts(2023, 6, 3, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_specific_st_never_stop_ten_year_after() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::SpecificWeekday,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0b00000001, //String::from("St"),
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    // executou no ultimo sab dia 5
    // a executar após qua ás 10
    let _res = find_next_event(CtrlTime::from_utc_parts(2033, 6, 18, 10, 0, 0), &schedule);
    // o sab seguinte a dia 5 é dia 12
    let target = CtrlTime::from_utc_parts(2033, 6, 25, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}
//testar stop condition x-retries e date
// x - retries - testar os varios branchs no limite do retry e com o mesmo expirado

//este é suposto correr se estiver no seu momemto, e não correr em mais momento nenhum
//ou seja, get next event devolve o start ts se for maior do que o after, ou devolve none
#[test]
fn test_sched_run_never_repeat_ok() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 4, 10, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

//no limite do retry - deve correr
#[test]
fn test_sched_run_repeat_daily_in_the_limit() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Retries,
        stop_retries: 2,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 1,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 10, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

//para lá  do retry - deve dar none
#[test]
fn test_sched_run_daily_after_last_none() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Retries,
        stop_retries: 2,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 2,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 10, 0, 0), &schedule);
    if let Some(new_time) = _res.unwrap() {
        panic!("devolveu {} e devia ser none", new_time.as_rfc3339_str_e());
    } else {
        println!("devolveu None como devia");
    }
}

//este é suposto correr se estiver no seu momemto, e não correr em mais momento nenhum
//ou seja, get next event devolve o start ts se for maior do que o after, ou devolve none
#[test]
fn test_sched_run_never_repeat_none() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::default(),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 10, 0, 0), &schedule);
    if let Some(new_time) = _res.unwrap() {
        panic!("devolveu {} e devia ser none", new_time.as_rfc3339_str_e());
    } else {
        println!("devolveu None como devia");
    }
}

// date - testar os varios branchs no limite da data (deve dar sucesso ) e depois da data (deve dar None)
#[test]
fn test_sched_run_repeat_daily_stop_date_in_the_limit() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Date,
        stop_retries: 2,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 1,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 1,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 10, 0, 0), &schedule);
    let target = CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0);
    if let Some(new_time) = _res.unwrap() {
        let diff = ((new_time.0 - target.0) as i64).abs();
        // println!("diff:{}", diff);
        // println!("Last Run: {} Truth Next Run: {}  Calc Next Run: {}", schedule.start.as_rfc3339_str_e(), target.as_rfc3339_str_e(), new_time.as_rfc3339_str_e());
        assert!(diff == 0, "{}", msg(schedule.start, new_time));
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_repeat_daily_stop_date_none() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    let _res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 6, 10, 0, 0), &schedule);
    if let Some(new_time) = _res.unwrap() {
        panic!("devolveu {} e devia ser none", new_time.as_rfc3339_str_e());
    } else {
        println!("devolveu None como devia");
    }
}

// #[cfg(test)]
// colocar
fn msg(old: CtrlTime, new: CtrlTime) -> String {
    format!("temos um sched a começar em {} e o proximo evento é em {}", old.as_rfc3339_str_e(), new.as_rfc3339_str_e())
}

#[test]
fn test_schedule_seconds() {
    let mut start_ts = CtrlTime(1000);
    let mut nsecs: u64 = 5;
    let new_time = aux(start_ts, nsecs);
    assert!(new_time == start_ts.add_secs(nsecs), "temos um sched a começar em {} e o proximo evento é em {}", start_ts, start_ts.add_secs(nsecs));

    start_ts = CtrlTime(10000);
    nsecs = 10;
    let new_time = aux(start_ts, nsecs);
    assert!(new_time == start_ts.add_secs(nsecs), "temos um sched a começar em {} e o proximo evento é em {}", start_ts, start_ts.add_secs(nsecs));

    start_ts = CtrlTime(1000);
    nsecs = 300;
    let new_time = aux(start_ts, nsecs);
    assert!(new_time == start_ts.add_secs(nsecs), "temos um sched a começar em {} e o proximo evento é em {}", start_ts, start_ts.add_secs(nsecs));

    start_ts = new_time;
    nsecs = 65000;
    let new_time = aux(start_ts, nsecs);
    assert!(new_time == start_ts.add_secs(nsecs), "temos um sched a começar em {} e o proximo evento é em {}", start_ts, start_ts.add_secs(nsecs));
}

fn aux(start_ts: CtrlTime, interval: u64) -> CtrlTime {
    let mut schedule = Schedule {
        start: start_ts,
        repeat_kind: ScheduleRepeat::Every, // never/hourly/daily/weekday/specific week days/weekly/monthly/every   // 8        16
        stop_condition: ScheduleStop::Never, // "never", #never, x-retries, date                                     // 8        24
        stop_retries: 0,                    // 2
        stop_date_ts: CtrlTime::default(),  // 8        34
        repeat_spec_wd: 0,                  //"", Sunday|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday          // 24 + 9   67
        repeat_every_qty: interval as u16,  // 2
        repeat_every_unit: ScheduleRepeatUnit::Seconds, // "", minutes, hours, days, week, month                            // 8
        retries_count: 0,                   // 2        87
    };

    let _res = schedule.set_next_event();
    if let Some(new_time) = _res.unwrap() {
        new_time
    } else {
        panic!("não devolveu Some(xxx) mas devia");
    }
}

#[test]
fn test_sched_run_err_repeat_kind() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 6, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Seconds,
        retries_count: 0,
    };
    let res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 6, 10, 0, 0), &schedule);
    if let Err(ScheduleError::RepeatKindWithNoQty { repeat_kind: _ }) = res {
        assert!(true);
    }
}

#[test]
fn test_sched_run_start_eq_stop() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Every,
        stop_condition: ScheduleStop::Date,
        stop_retries: 0,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 2,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };
    let res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0), &schedule);
    if let Ok(Some(time)) = res {
        assert!(time == schedule.start);
    }
}

#[test]
fn test_sched_run_start_still_to_arrive() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };
    //alimentamos com uma data anterior ao start
    let res = find_next_event(CtrlTime::from_utc_parts(2022, 3, 5, 8, 0, 0), &schedule);
    if let Ok(Some(time)) = res {
        assert!(time == schedule.start);
    }
}

#[test]
fn test_sched_run_is_expired() {
    let schedule = Schedule {
        start: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_kind: ScheduleRepeat::Never,
        stop_condition: ScheduleStop::Never,
        stop_retries: 0,
        stop_date_ts: CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0),
        repeat_spec_wd: 0,
        repeat_every_qty: 0,
        repeat_every_unit: ScheduleRepeatUnit::Days,
        retries_count: 0,
    };

    let time = CtrlTime::from_utc_parts(2022, 3, 5, 10, 0, 0);
    
    assert!(!schedule.is_time_to_run(time));
}

#[test]
fn test_every_seconds(){
    // let mut c = d.benchmark_group("bench_schedule");
    // c.bench_function("bench_schedule_base_struct_creation", |b| b.iter(|| bench_schedule_base_struct_creation()));
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

    sched.start = CtrlTime::from_utc_parts(2022, 3, 9, 9, 0, 0);
    sched.repeat_kind = ScheduleRepeat::Never;
    sched.repeat_every_unit = ScheduleRepeatUnit::Seconds;
    
    let _schedule = sched.get_next_event();
}