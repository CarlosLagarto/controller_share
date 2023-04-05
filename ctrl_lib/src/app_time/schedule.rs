use crate::app_time::{ctrl_time::*, schedule_params::*, sunrise::*};
use crate::config::geo_pos::*;
use ctrl_prelude::globals::*;

const AUX: [u64; 8] = [
    0,
    CtrlTime::NR_NANOS_IN_A_DAY,
    CtrlTime::NR_NANOS_IN_A_DAY * 2,
    CtrlTime::NR_NANOS_IN_A_DAY * 3,
    CtrlTime::NR_NANOS_IN_A_DAY * 4,
    CtrlTime::NR_NANOS_IN_A_DAY * 5,
    CtrlTime::NR_NANOS_IN_A_DAY * 6,
    CtrlTime::NR_NANOS_IN_A_DAY * 7,
];

pub const NR_NANOS: [u64; 5] = [
    GIGA_U,                       //SCHEDULE_REPEAT_UNIT::seconds
    60 * GIGA_U,                  //SCHEDULE_REPEAT_UNIT::minutes
    CtrlTime::NR_NANOS_IN_A_HOUR, //SCHEDULE_REPEAT_UNIT::hours
    CtrlTime::NR_NANOS_IN_A_DAY,  //SCHEDULE_REPEAT_UNIT::days
    CtrlTime::NR_NANOS_IN_A_WEEK, //SCHEDULE_REPEAT_UNIT::weeks
];

#[repr(u8)]
pub enum WeekDaysBC {
    Sunday = 0b01000000,
    Monday = 0b00100000,
    Tuesday = 0b00010000,
    Wednesday = 0b00001000,
    Thursday = 0b00000100,
    Friday = 0b00000010,
    Saturday = 0b00000001,
}

/// Dimension = 32
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    pub start: CtrlTime,
    pub repeat_kind: ScheduleRepeat,  // never/specific week days/every
    pub stop_condition: ScheduleStop, // "never", x-retries, date
    pub stop_retries: u16,
    pub stop_date_ts: CtrlTime,
    ///  : 'Sunday'  | 'Monday' |'Tuesday' |'Wednesday'|'Thursday'| 'Friday' |'Saturday'
    ///  ------------|----------|----------|-----------|----------|----------|-----------
    ///  : 0b01000000|0b00100000|0b00010000|0b00001000 |0b00000100|0b00000010|0b00000001
    pub repeat_spec_wd: u8,
    pub repeat_every_qty: u16,
    pub repeat_every_unit: ScheduleRepeatUnit, // "", minutes, hours, days, week, month
    pub retries_count: u16,
}

impl Schedule {
    #[inline]
    pub fn build_run_forever(start: CtrlTime, repeat_every_qty: u16, repeat_every_unit: ScheduleRepeatUnit) -> Self {
        Self {
            start,
            repeat_kind: ScheduleRepeat::Every,
            stop_condition: ScheduleStop::Never,
            stop_retries: 0,
            stop_date_ts: CtrlTime(0),
            repeat_spec_wd: 0,
            repeat_every_qty,
            repeat_every_unit,
            retries_count: 0,
        }
    }

    #[inline]
    pub fn build_run_once(start_ts: CtrlTime) -> Self {
        Self {
            start: start_ts,
            repeat_kind: ScheduleRepeat::Never,
            stop_condition: ScheduleStop::Retries,
            stop_retries: 1,
            stop_date_ts: CtrlTime(0),
            repeat_spec_wd: 0,
            repeat_every_qty: 0,
            repeat_every_unit: ScheduleRepeatUnit::Seconds, // for the repeat_kind 'never' is irrelevant
            retries_count: 0,
        }
    }
}

impl Schedule {
    #[cfg_attr(dev, dev)]
    pub fn get_next_event(&self) -> Result<Option<CtrlTime>, ScheduleError> {
        find_next_event(self.start, self)
    }

    #[cfg_attr(dev, dev)]
    pub fn set_next_event(&mut self) -> Result<Option<CtrlTime>, ScheduleError> {
        let res = find_next_event(self.start, self);
        if let Ok(Some(next_ts)) = res {
            self.start = next_ts;
            res
        } else {
            Ok(None)
        }
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_time_to_run(&self, time: CtrlTime) -> bool {
        
        if time < self.start { return false; } // most used path.  avoids executing unnecessary code

        let have_stop_condition = self.stop_condition != ScheduleStop::Never;
        let have_retries = self.stop_condition == ScheduleStop::Retries;
        let have_stop_date = self.stop_condition == ScheduleStop::Date;
        
        let retries_exceeded = self.retries_count >= self.stop_retries;
        let have_stop_date_condition = have_stop_condition && have_stop_date;

        if is_expired(self, time, have_retries, retries_exceeded, have_stop_date_condition) {
            return false;
        }
        // at the end avoid one if, and return the expression result
        // if self.retries_count >= self.stop_retries {
        //     return false;
        // }
        // true
        // self.retries_count >= self.stop_retries
        // this expression was in memory - by coincidence.  Does not help code legebility, but its logiclly correct
        retries_exceeded
    }
}

/// find and return next event time after after_time.  Return None if no event,
///
/// Assume utc time. Its caller responsability to convert to/from utc
///
/// Schedule Parameter:
///    - start_ts -- time to start the event, or the last time the event ran
///    - repeat_kind -- never/specific week day/every
///    - stop_condition --  never/x-retries/date
///    - stop_retries -- nr of runs/retries
///    - stop_date_ts -- stop date used when stop_condition is date
///    - repeat_spec_wd -- Su|Mo|Tu|We|Th|Fr|St
///    - repeat_every_qty --  nr of runs/repetitions
///    - repeat_every_unit -- seconds, minutes, hours, days, week
///    - repeat_retries_count -- nr of runs already executed<br>
/// <br>
/// after_time Parameter -- find the time after after_time. Tipically this time is the last executed run.<br>
/// 
/// Usually to be called after the execution of the task/event
/// Updating the next run and retires number is caller responsability
///
/// SCHEDULE_REPEAT_UNIT it's only relevant/considered for SCHEDULE_REPEAT::every
///
#[rustfmt::skip]
pub fn find_next_event(after_time: CtrlTime, schedule: &Schedule) -> Result<Option<CtrlTime>, ScheduleError> {
    // Guards for data coherency - just in case the programmer is distracted
    if schedule.repeat_kind != ScheduleRepeat::Never && schedule.repeat_every_qty == 0 {
        return Err(ScheduleError::RepeatKindWithNoQty { repeat_kind: String::from(schedule.repeat_kind.description()) });
    }

    let have_stop_condition = schedule.stop_condition != ScheduleStop::Never;
    let have_retries = schedule.stop_condition == ScheduleStop::Retries;
    let have_stop_date = schedule.stop_condition == ScheduleStop::Date;
    let retries_exceeded = schedule.retries_count >= schedule.stop_retries;
    let have_stop_date_condition = have_stop_condition && have_stop_date;

    if is_expired(schedule, after_time, have_retries, retries_exceeded, have_stop_date_condition) {
        return Ok(None);
    }

    let valid_ts = |ts: CtrlTime| -> Option<CtrlTime> {
        let result = [None, Some(ts)];
        result[(ts > after_time && (!have_stop_date || ts <= schedule.stop_date_ts)) as usize]
    };

    // validates if start datetime is valid for the current conditions, i.e., if start = after and no stop date
    // (worst case, where start is valid, and one may call get next before the time to start)
    // all other situation we need to find the next start
    match valid_ts(schedule.start) {
        Some(_) => {
            return Ok(Some(schedule.start));
        }
        _ if have_stop_date_condition && schedule.start == schedule.stop_date_ts => {
            return Ok(Some(schedule.start));
        }
        _ => {}
    }

    let next_start: Option<CtrlTime>;

    match schedule.repeat_kind {
        ScheduleRepeat::Every => {
            let repetion_period: i64 = schedule.repeat_every_qty as i64 * NR_NANOS[schedule.repeat_every_unit as usize] as i64;
            // if after_ts < start_ts, take start_ts, i.e., distance =  0
            let distance: i64 = (after_time.0 - schedule.start.0).max(0) as i64; 
            let occurrences = (distance / repetion_period) + 1;
            next_start = Some(schedule.start + (occurrences * repetion_period));
        }

        ScheduleRepeat::SpecificWeekday => {
            let mut candidate: u64 = 0;
            let mut result: u64 = CtrlTime::MAX;

            let day = schedule.start.week_day_e() as usize;
            // the lambda, performance wise, is the compromise between duplicating the code (more performance) and a funtion (less code but less performant)
            // compiler seems to make better optimizations with lambda functions
            let mut lambda = |diff: u64| {
                candidate = schedule.start.0 + diff;
                if candidate < after_time.0 {
                    candidate += (((after_time.0 - candidate) / CtrlTime::NR_NANOS_IN_A_WEEK) + 1) * CtrlTime::NR_NANOS_IN_A_WEEK;
                }
                result = result.min(candidate);
            };
            //--- loop unroll experiment -- less 100 ns than the code with a loop - se history in git
            if (0b01000000 & schedule.repeat_spec_wd) == 0b01000000 { lambda(AUX[TBL_DIFFS[day][0] as usize]); }
            if (0b00100000 & schedule.repeat_spec_wd) == 0b00100000 { lambda(AUX[TBL_DIFFS[day][1] as usize]); }
            if (0b00010000 & schedule.repeat_spec_wd) == 0b00010000 { lambda(AUX[TBL_DIFFS[day][2] as usize]); }
            if (0b00001000 & schedule.repeat_spec_wd) == 0b00001000 { lambda(AUX[TBL_DIFFS[day][3] as usize]); }
            if (0b00000100 & schedule.repeat_spec_wd) == 0b00000100 { lambda(AUX[TBL_DIFFS[day][4] as usize]); }
            if (0b00000010 & schedule.repeat_spec_wd) == 0b00000010 { lambda(AUX[TBL_DIFFS[day][5] as usize]); }
            if (0b00000001 & schedule.repeat_spec_wd) == 0b00000001 { lambda(AUX[TBL_DIFFS[day][6] as usize]); }
            next_start = Some(CtrlTime(result));
        }        
        ScheduleRepeat::Never => next_start = Some(schedule.start),
    }

    if next_start.is_none() { return Ok(None); }

    // lastly validate if the found date is after the stop date, if the schedule have a stop date condition
    Ok(valid_ts(next_start.unwrap()))
}

#[inline]
pub fn adjust_start_date_to_sunrise(geo_pos: &GeoPos, time_date: CtrlTime) -> CtrlTime {
    // adjust reference date parameter, so the event finish before the sunrise at the reference date
    let times = sun_times(time_date, geo_pos.lat, geo_pos.long, geo_pos.elev);
    times.0 - 117u64 * CtrlTime::NR_NANOS_IN_A_MINUTE // 117 its the magic number determined by the sum of the maximum time for each sector
}

// DESIGN NOTE 
// when exacly to run the task if the time to run it already passed?
// During normal execution, tasks are scheduled to run somewhere in the future.
// The issue is when, for some reason (some delay, or just program was stopped and we run it after the start time), and we test the start time some time after the right moment.
// What is the right interval to "catch" the time?  
// It may make sense to validate an interval, say around 5 seconds.
// As I still do not have a goot rational for this, for now it just test if the current time is greater then the start time, and run the task.
#[inline] 
pub fn is_expired(schedule: &Schedule, after: CtrlTime, have_retries: bool, retries_exceeded: bool, have_stop_date_condition: bool) -> bool {
    schedule.repeat_kind == ScheduleRepeat::Never && schedule.start < after
        || have_retries && retries_exceeded
        || have_stop_date_condition && schedule.start > schedule.stop_date_ts  //previously >=
}