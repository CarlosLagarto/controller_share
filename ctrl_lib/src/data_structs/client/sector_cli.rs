use serde::{Deserialize, Serialize};

use crate::services::electronics::valve_state::*;
use crate::services::irrigation::{sector::*, sector_run::*};
use crate::app_time::ctrl_time::*;
use crate::data_structs::{client::sync_op::*, rega::watering_status::*};
use ctrl_prelude::domain_types::*;

/// Dimension 112
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SectorCli {
    pub desc: String, 
    pub name: String, 
    pub last_watered_in: UTC_UNIX_TIME,
    pub last_change: UTC_UNIX_TIME,
    pub deficit: f32,
    pub percolation: f32,  // mm/minute
    pub debit: f32,        // mm/minute
    pub max_duration: f32, // minutes
    pub stress_perc: f32,
    pub stress_score: u8,

    pub id: SECTOR_ID,
    pub enabled: bool,
    pub op: SyncOp,

    pub minutes_to_water: f32,
    pub start: UTC_UNIX_TIME,
    pub end: UTC_UNIX_TIME,
    pub status: WateringStatus,
}

impl SectorCli {
    #[inline]
    pub fn to_client(sec: &Sector, sec_run: &SectorRun) -> SectorCli {
        SectorCli {
            desc: sec.desc.clone(),
            name: sec.name.clone(),
            last_watered_in: sec.last_watered_in.ux_ts(),
            last_change: sec.last_change.ux_ts(),
            deficit: sec.deficit,
            percolation: sec.percolation,
            debit: sec.debit,
            max_duration: sec.max_duration,
            stress_perc: sec.stress_perc,
            stress_score: sec.stress_score,
            id: sec.id,
            enabled: sec.enabled,
            op: sec.op.clone(),

            minutes_to_water: sec_run.wtr_tgt_min,
            start: sec_run.start.ux_ts(),
            end: sec_run.end.ux_ts(),
            status: sec_run.status,
        }
    }
    #[inline]
    pub fn from_client(&self) -> Sector {
        Sector {
            desc: self.desc.clone(),
            name: self.name.clone(),
            last_watered_in: CtrlTime::from_ux_ts(self.last_watered_in), //REVIEW isto será para validar porque o cliente não controla isto
            last_change: CtrlTime::from_ux_ts(self.last_change),
            deficit: self.deficit,
            percolation: self.percolation,
            debit: self.debit,
            max_duration: self.max_duration,
            stress_perc: self.stress_perc,
            stress_score: self.stress_score,
            id: self.id,
            enabled: self.enabled,
            op: self.op.clone(),
            update_db: true,
            state: RelayState::Closed,
            device_id: u16::MAX,
        }
    }
}
