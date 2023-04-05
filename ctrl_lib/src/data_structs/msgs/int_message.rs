use strum_macros::Display;

use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::{alert::*, ext_message::*, log_error::*, weather::*};
use crate::data_structs::rega::command::*;
use crate::services::electronics::{actuator::*, devices_and_scenes::*};
use crate::services::irrigation::wtr_history::*;

/// Dimension 160
#[derive(Clone, Debug)]
pub struct IntMessage {
    pub data: MsgData,
    pub time: CtrlTime,
}

impl IntMessage {
    #[inline]
    pub fn build(tipo: MsgData, time: CtrlTime) -> IntMessage {
        IntMessage {
            data: tipo,
            time,
        }
    }
}

/// Dimension 152
#[derive(Clone, Debug, Display)]
pub enum MsgData {
    /// water machine command msgs
    Command(Command),
    /// msg to terminate program - controller
    ShutDown(CtrlTime),
    /// weather info
    Weather(Weather),
    /// External message received
    MessageIn(ExtMsgIn),
    /// External msg to send
    MessageOut(ExtMsgOut),
    /// Alert msg ( for now only weather have this use case, but others could be possible)
    Alert(Alert),
    /// Water machine state changed
    StateChanged,
    /// Cycle added msg
    CycleAdded, //isto é interno ao COMMAND
    /// SPRINT SENSORES - future usage
    SensorData,
    /// Just letting the world know that the routine started.  No use for now...
    DBMaintStarted,
    /// Just letting the world know that the routine is completed.  No use for now...
    DBMaintCompleted,
    /// Just letting the world know that the water machine started.  No use for now...
    WaterMachineStarted,
    /// Just letting the world know that the water machine stopped.  No use for now...
    WaterMachineStopped,
    /// Signals the client manager that there are errors to send to the clients.
    ClientError(LogError),
    /// Client requests for weather history
    GetWeatherHistory,
    /// Client request for the full BD
    GetFullDB,
    /// Client info to sync DB
    SyncDB,
    /// Client request for watering history
    GetWateringHistory,
    /// watering history response
    RspWateringHistory(WaterHstry),
    /// Client request for the devices and scenes
    GetDevicessAndScenes,
    /// Devices and scenes response
    RspDevicesAndScenes(DevicesAndScenesData),
    /// Client command for device or scene
    SetActuatorOrScene(ActuatorData),
    /// device or scene command response
    RspActuatorOrScene(ActuatorData),
    /// TODO
    Shellies(String, String),
}

impl MsgData {
    // This have always to be in sync with MsgType - programmer responsability
    #[inline]
    pub fn tipo(&self) -> usize {
        match self {
            MsgData::Command(_) => 0,
            MsgData::ShutDown(_) => 1,
            MsgData::Weather(_) => 2,
            MsgData::MessageIn(_) => 3,
            MsgData::MessageOut(_) => 4,
            MsgData::Alert(_) => 5,
            MsgData::StateChanged => 6,
            MsgData::CycleAdded => 7,
            MsgData::SensorData => 8,
            MsgData::DBMaintStarted => 9,
            MsgData::DBMaintCompleted => 10,
            MsgData::WaterMachineStarted => 11,
            MsgData::WaterMachineStopped => 12,
            MsgData::ClientError(_) => 13,
            MsgData::GetWeatherHistory => 14,
            MsgData::GetFullDB => 15,
            MsgData::SyncDB => 16,
            MsgData::GetWateringHistory => 17,
            MsgData::RspWateringHistory(_) => 18,
            MsgData::GetDevicessAndScenes => 19,
            MsgData::RspDevicesAndScenes(_) => 20,
            MsgData::SetActuatorOrScene(_) => 21,
            MsgData::RspActuatorOrScene(_) => 22,
            MsgData::Shellies(_, _) => 23,
        }
    }
}

/// Used in broker and MQTT services
#[derive(Display, Clone, Copy)]
#[repr(u8)]
pub enum MsgType {
    Command = 0,
    ShutDown = 1,
    Weather = 2,
    MessageIn = 3,
    MessageOut = 4,
    Alert = 5,
    StateChanged = 6,
    CycleAdded = 7,
    SensorData = 8,
    DBMaintStarted = 9,
    DBMaintCompleted = 10,
    WaterMachineStarted = 11,
    WaterMachineStopped = 12,
    ClientError = 13,
    GetWeatherHistory = 14,
    GetFullDB = 15,
    SyncDB = 16,
    GetWateringHistory = 17,
    RspWateringHistory = 18,
    GetDevicessAndScenes = 19,
    RspDevicesAndScenes = 20,
    SetActuatorOrScene = 21,
    RspActuatorOrScene = 22,
    Shellies = 23
}

/// used in broker
pub const COUNT_EVENT_TYPE: usize = 24;
