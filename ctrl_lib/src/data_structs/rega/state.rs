use serde::{self, Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Display, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, EnumString)]
#[repr(u8)]
pub enum State {
    #[default]
    Starting = 0,
    NoScheduleDef = 1,
    EstablishMode = 2,
    ManWait = 3,
    WzrWait = 4,
    StdWait = 5,
    ManWtrCycle = 6,
    StdWtrCycle = 7,
    WzrWtrCycle = 8,
    ManWtrSector = 9,
    StdWtrSector = 10,
    WzrWtrSector = 11,
    ManWtrSectorDirect = 12,
    SuspendedWizard = 13,
    Error = 14,
    Shutdown = 15,
}

pub const STATE_COUNT: u8 = 16;
