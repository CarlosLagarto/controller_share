use serde::{Deserialize, Serialize};

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{ext_message::*, topic::*};

/// Dimension 88
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogError {
    pub header: Option<Header>,
    pub error: String,
}

impl LogError {
    #[inline]
    pub fn new_out(error: String, time: CtrlTime) -> ExtMsgOut {
        ExtMsgOut::new(
            Topic::STC_SND_LOG_ERROR,
            ExtMsgOut::LogError(LogError {
                header: None,
                error,
            }),
            time,
        )
    }
}
