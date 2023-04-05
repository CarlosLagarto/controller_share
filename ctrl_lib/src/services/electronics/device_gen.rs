use serde::{Deserialize, Serialize};

use crate::db::SqlRow;
use crate::services::electronics::actuator::*;
use crate::string_concat::*;
use ctrl_prelude::domain_types::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceGen {
    pub id: DEVICE_ID,  // 65000 devices will be more than enough
    pub identifier: String,
    pub status: bool,
    pub ip: String,
    pub cmd_on: String,
    pub cmd_off: String,
    pub get_status: String,

    pub desc: String,
    pub cmd_up: String,
    pub cmd_stop: String,
    pub cmd_down: String,
    pub shutter_get_status: String,
    pub device_type: DeviceType,
}

impl DeviceGen {
    #[inline]
    pub fn get_cmd(&self, cmd: ActuatorCommand) -> String {
        let mut url: String = String::from("");
        match cmd {
            ActuatorCommand::Off => url = string_concat!("http://", self.ip, "/", self.cmd_off),
            ActuatorCommand::On => url = string_concat!("http://", self.ip, "/", self.cmd_on),
            ActuatorCommand::Up => url = string_concat!("http://", self.ip, "/", self.cmd_up),
            ActuatorCommand::Stop => url = string_concat!("http://", self.ip, "/", self.cmd_stop),
            ActuatorCommand::Down => url = string_concat!("http://", self.ip, "/", self.cmd_down),
            _ => (),
        }
        url
    }
}

impl From<&SqlRow<'_>> for DeviceGen {
    #[inline]
    fn from(sql_row: &SqlRow) -> DeviceGen {
        let sql_row = sql_row;

        DeviceGen {
            id: sql_row.get(0).unwrap(),
            identifier: sql_row.get(1).unwrap(),
            status: sql_row.get(2).unwrap(),
            ip: sql_row.get(3).unwrap(),
            cmd_on: sql_row.get(4).unwrap(),
            cmd_off: sql_row.get(5).unwrap(),
            get_status: sql_row.get(6).unwrap(),
            desc: sql_row.get(7).unwrap(),
            cmd_up: sql_row.get(8).unwrap(),
            cmd_stop: sql_row.get(9).unwrap(),
            cmd_down: sql_row.get(10).unwrap(),
            shutter_get_status: sql_row.get(11).unwrap(),
            device_type: unsafe { DeviceType::from_unchecked(sql_row.get(12).unwrap()) },
        }
    }
}
