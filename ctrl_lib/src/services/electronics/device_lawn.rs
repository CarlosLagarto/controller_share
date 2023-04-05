use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{db::SqlRow, services::electronics::actuator::*, string_concat::*};
use ctrl_prelude::domain_types::*;

pub static mut NR_OF_WATER_DEVICES: u16 = 0;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeviceLawn {
    pub id: DEVICE_ID, // 65000 devices will be more than enough
    pub identifier: String,
    pub status: bool,
    pub ip: String,
    pub cmd_on: String,
    pub cmd_off: String,
    pub get_status: String,
}

pub type DeviceWateringList = FxHashMap<DEVICE_ID, DeviceLawn>;

impl DeviceLawn {
    #[inline]
    pub fn get_cmd(&self, cmd: ActuatorCommand) -> String {
        let mut url: String = String::from("");
        match cmd {
            ActuatorCommand::Off => url = string_concat!("http://", self.ip, "/", self.cmd_off),
            ActuatorCommand::On => url = string_concat!("http://", self.ip, "/", self.cmd_on),
            // ActuatorCommand::Up => url = string_concat!("http://", self.ip, "/", self.cmd_up),
            // ActuatorCommand::Stop => url = string_concat!("http://", self.ip, "/", self.cmd_stop),
            // ActuatorCommand::Down => url = string_concat!("http://", self.ip, "/", self.cmd_down),
            _ => (),
        }
        url
    }
}

impl From<&SqlRow<'_>> for DeviceLawn {
    #[inline]
    fn from(sql_row: &SqlRow) -> DeviceLawn {
        let sql_row = sql_row;

        DeviceLawn {
            id: sql_row.get(0).unwrap(),
            identifier: sql_row.get(1).unwrap(),
            status: sql_row.get(2).unwrap(),
            ip: sql_row.get(3).unwrap(),
            cmd_on: sql_row.get(4).unwrap(),
            cmd_off: sql_row.get(5).unwrap(),
            get_status: sql_row.get(6).unwrap(),
        }
    }
}
