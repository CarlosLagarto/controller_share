use serde::{Deserialize, Serialize};

use crate::app_time::{ctrl_time::*, schedule::Schedule, schedule_params::*};
use ctrl_prelude::domain_types::UTC_UNIX_TIME;

/// Dimension = 32
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScheduleCli {
    pub start: UTC_UNIX_TIME,
    pub repeat_kind: ScheduleRepeat,  // never/specific week days/every
    pub stop_condition: ScheduleStop, // "never", x-retries, date
    pub stop_retries: u16,
    pub stop_date_ts: UTC_UNIX_TIME,
    ///  : 'Sunday'  | 'Monday' |'Tuesday' |'Wednesday'|'Thursday'| 'Friday' |'Saturday'
    ///  ------------|----------|----------|-----------|----------|----------|-----------
    ///  : 0b01000000|0b00100000|0b00010000|0b00001000 |0b00000100|0b00000010|0b00000001
    pub repeat_spec_wd: u8,
    pub repeat_every_qty: u16,
    pub repeat_every_unit: ScheduleRepeatUnit, // "", minutes, hours, days, week, month
    pub retries_count: u16,
}

impl ScheduleCli {
    #[inline]
    pub fn to_client(schedule: &Schedule) -> ScheduleCli {
        ScheduleCli {
            start: schedule.start.ux_ts(),
            repeat_kind: schedule.repeat_kind,
            stop_condition: schedule.stop_condition,
            stop_retries: schedule.stop_retries,
            stop_date_ts: schedule.stop_date_ts.ux_ts(),
            repeat_spec_wd: schedule.repeat_spec_wd,
            repeat_every_qty: schedule.repeat_every_qty,
            repeat_every_unit: schedule.repeat_every_unit,
            retries_count: schedule.retries_count,
        }
    }

    #[inline]
    pub fn from_client(&self) -> Schedule {
        Schedule {
            start: CtrlTime::from_ux_ts(self.start),
            repeat_kind: self.repeat_kind,
            stop_condition: self.stop_condition,
            stop_retries: self.stop_retries,
            stop_date_ts: CtrlTime::from_ux_ts(self.stop_date_ts),
            repeat_spec_wd: self.repeat_spec_wd,
            repeat_every_qty: self.repeat_every_qty,
            repeat_every_unit: self.repeat_every_unit,
            retries_count: self.retries_count,
        }
    }
}
