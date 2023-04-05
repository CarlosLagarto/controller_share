use serde::{Deserialize, Serialize};

// use crate::{
use crate::app_time::ctrl_time::*;
use crate::data_structs::client::{cycle_run_cli::*, schedule_cli::*, sync_op::*};
use crate::services::irrigation::{cycle::*, cycle_type::*};

use ctrl_prelude::domain_types::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CycleCli {
    #[serde(flatten)]
    pub run: CycleRunCli,
    #[serde(flatten)]
    pub schedule: ScheduleCli,
    pub name: String, 
    pub last_change: UTC_UNIX_TIME,
    pub last_run: UTC_UNIX_TIME,
    pub op: SyncOp,
    pub sunrise_flg: SUN_FLAG,
    pub sunset_flg: SUN_FLAG,

    pub cycle_type: u8,
}

impl CycleCli {
    #[inline]
    pub fn from_client(&self) -> Cycle {
        Cycle {
            run: self.run.from_client(),
            schedule: self.schedule.from_client(),
            last_run: CtrlTime::from_ux_ts(self.last_run),
            sunrise_flg: self.sunrise_flg,
            sunset_flg: self.sunset_flg,
            cycle_type: unsafe { CycleType::from_unchecked(self.cycle_type) },
            name: self.name.clone(),
            last_change: CtrlTime::from_ux_ts(self.last_change),
            op: self.op.clone(),
            ptr: None,
        }
    }

    #[inline]
    pub fn to_client(cycle: &Cycle) -> CycleCli {
        CycleCli {
            run: CycleRunCli::to_client(&cycle.run),
            schedule: ScheduleCli::to_client(&cycle.schedule),
            last_run: cycle.last_run.ux_ts(),
            sunrise_flg: cycle.sunrise_flg,
            sunset_flg: cycle.sunset_flg,
            cycle_type: cycle.cycle_type as u8,
            name: cycle.name.clone(),
            last_change: cycle.last_change.ux_ts(),
            op: cycle.op.clone(),
        }
    }
}
