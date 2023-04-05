use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{ext_message::*, topic::*};

#[derive(Display, Clone)]
#[repr(u8)]
pub enum CONNECTION {
    ONLINE,
    OFFLINE,
}

/// Dimension 88
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Connection {
    pub header: Option<Header>,
    pub status: String,
}

impl Connection {
    #[inline]
    pub fn new_out(connection_status: bool, time: CtrlTime) -> ExtMsgOut {
        let status = if connection_status { CONNECTION::ONLINE.to_string() } else { CONNECTION::OFFLINE.to_string() };
        let conn = Connection {
            status,
            header: None,
        };
        ExtMsgOut::new(Topic::SERVER_CONNECTION, ExtMsgOut::Connection(conn), time)
    }
}
