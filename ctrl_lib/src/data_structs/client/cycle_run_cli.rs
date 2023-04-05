use serde::{Deserialize, Serialize};

use crate::{app_time::ctrl_time::*, data_structs::rega::watering_status::*, services::irrigation::cycle_run::*};
use ctrl_prelude::domain_types::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CycleRunCli {
    #[serde(rename = "run_start")]
    pub start: UTC_UNIX_TIME,
    pub end: UTC_UNIX_TIME,
    pub cycle_id: CYCLE_ID,
    pub run_id: u32,
    pub status: WateringStatus,
}

impl CycleRunCli {
    #[inline]
    pub fn from_client(&self) -> CycleRun {
        CycleRun {
            start: CtrlTime::from_ux_ts(self.start),
            end: CtrlTime::from_ux_ts(self.end),
            cycle_id: self.cycle_id,
            run_id: self.run_id,
            status: self.status,
            ..Default::default()
        }
    }

    #[inline]
    pub fn to_client(cycle_run: &CycleRun) -> CycleRunCli {
        CycleRunCli {
            start: cycle_run.start.ux_ts(),
            end: cycle_run.end.ux_ts(),
            cycle_id: cycle_run.cycle_id,
            run_id: cycle_run.run_id,
            status: cycle_run.status,
        }
    }
}
