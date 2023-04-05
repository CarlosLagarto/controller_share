use serde::{self, Deserialize, Serialize};

use crate::app_time::ctrl_time::*;

/// Dimensão = 16
#[derive(Clone, Serialize, Deserialize)]
pub enum TimerSignal {
    Shutdown,
    Stop,
    Timer(CtrlTime),
}

