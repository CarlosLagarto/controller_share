use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::watering_status::*;
use ctrl_prelude::domain_types::*;

/// Dimension: Stack = 56 bytes
#[derive(Clone, Default)]
pub struct SectorRun {
    pub wtr_tgt_min: f32,
    pub wtr_acc_min: f32,
    pub start: CtrlTime,
    pub end: CtrlTime,
    pub last_start: CtrlTime,
    pub cycle_id: CYCLE_ID,
    pub cycle_ptr: CYCLE_ID,
    pub curr_run: u32,
    pub sec_id: SECTOR_ID,
    pub skipped: bool,
    pub status: WateringStatus,
    pub upd_db: bool,
    pub is_manual: bool,
}

impl SectorRun {
    #[inline]
    pub fn new(cycle_id: CYCLE_ID, current_run: u32, sector_id: SECTOR_ID, status: WateringStatus) -> Self {
        Self {
            is_manual: false,
            cycle_id,
            curr_run: current_run,
            sec_id: sector_id,
            status,
            upd_db: true,
            ..Default::default()
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn is_watering(&self) -> bool { self.status == WateringStatus::Running }
}
