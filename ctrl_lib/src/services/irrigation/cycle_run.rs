use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::watering_status::*;
use ctrl_prelude::domain_types::*;

/// Dimension 32
#[derive(Clone, Debug, Default)]
pub struct CycleRun {
    pub start: CtrlTime,
    pub end: CtrlTime,
    pub cycle_id: CYCLE_ID,
    pub run_id: u32,
    pub status: WateringStatus,
    pub update_db: bool,
}
