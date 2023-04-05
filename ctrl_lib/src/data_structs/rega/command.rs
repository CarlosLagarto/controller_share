use strum_macros::Display;

use crate::data_structs::msgs::alert::*;
use crate::data_structs::rega::{mode::Mode, running_ptr::*};
use ctrl_prelude::domain_types::{CYCLE_ID, CYCLE_PTR, SECTOR_ID};

/// Dimension = 72
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, Display, PartialEq)]
pub enum Command {
    /// Start watering command
    Start,
    /// Change mode command.  Forces machine status to Establishmode 
    ChangeMode(Mode),
    /// Machine Stop command
    ShutDown,
    /// Start sector watering command
    StartSector,
    /// Start cycle command (the cycle in the param)
    StartCycle(CYCLE_PTR),
    /// Immediately stop .cycle command (the cycle in the param) 
    StopCycle(CYCLE_ID),
    /// Stop sector command 
    StopSector(RunningPtr),
    /// End cycle command 
    EndCycle(CYCLE_PTR),
    /// End sector command
    EndSector(RunningPtr),
    /// Forces sector watering
    ForceSector(SECTOR_ID),
    /// Forces cycle watering
    ForceCycle(CYCLE_ID),
    /// Water machine error state
    Error,
    /// Water machine suspended state (weather alert)
    Suspend(Alert),
    /// Resumes water machine
    Resume,
    /// Suspended state timeout
    ResumeTimeOut,
    /// Tells the machine that something has changed so it have an oportunity to refresh stuff and reconfigure
    /// It seems to overlap the functionality of ChangeMode
    ChangeState,
    /// Auxiliary state....runs the machine loop without doing nothing
    /// This was needed due to a tunning in the machine state machine handling...which may indicates that design could be improved 
    Null,
}
