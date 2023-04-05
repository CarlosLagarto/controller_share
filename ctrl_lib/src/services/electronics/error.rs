use ctrl_prelude::domain_types::DEVICE_ID;
use thiserror::*;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Error applying command on the relay. url: {} error: {}", 0, 1)]
    CommandError(String, String),
    // #[error("Error turning off the relay. url: {} error: {}", 0, 1)]
    // TurnOffError(String, String),
    #[error("Issue processing url result: {}, with error: {}", 0, 1)]
    ProcessingResult(String, String),
    #[error("Device not in the DB: {}", 0)]
    DeviceNotRegistered(DEVICE_ID),
}