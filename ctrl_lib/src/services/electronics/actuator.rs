use num_enum::UnsafeFromPrimitive;
use strum_macros::Display;
use serde::{Deserialize, Serialize};

// Estes enums têm que estar alinhados com o que está definido no cliente
/// Actuator = 0 <br>
/// Scene = 1<br>
#[derive(Clone, Copy, Debug, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum ActuatorType {
    Actuator = 0,
    Scene = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum DeviceType {
    Relay = 0,
    Roller = 1,
    TriggerSwitch = 2,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum ActuatorCommand {
    None = 1,
    Off = 2,
    On = 4,
    Up = 8,
    Stop = 16,
    Down = 32,
}

/// id: u16 <br>
/// actuator_type: ActuatorType<br>
/// cmd: ActuatorCommand<br>
/// status: String<br>
#[derive(Clone, Debug)]
pub struct ActuatorData {
    pub id: u16,
    pub actuator_type: ActuatorType,
    pub cmd: ActuatorCommand,
    pub status: Option<String>,
}
