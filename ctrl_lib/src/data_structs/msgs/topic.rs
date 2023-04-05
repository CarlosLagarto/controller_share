#![allow(non_camel_case_types)]
use std::fmt;

use serde::{self, Deserialize, Serialize};

use crate::utils::TESTING;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Topic {
    /// Server is sending the current weather
    STC_WEATHER = 0,
    /// Server is sending db update
    STC_SYNC_DB = 1,
    /// Server is sending the full db
    STC_SND_FULLDB = 2,
    /// Server is sending the weather alert
    STC_SND_ALERT = 3,
    /// Server is reseting the weather alert
    STC_SND_ALERT_RESET = 4,
    /// Server is sending the last 100 errors
    STC_SND_LOG_ERROR = 5,
    /// Server is sending the weather history
    STC_WEATHER_HIST = 6,
    /// Client is asking to stop a cycle watering
    CTS_STOP_CYCLE = 7,
    /// Client is asking to stop a sector watering
    CTS_STOP_SECTOR = 8,
    /// Client changed water machine mode
    CTS_STATUS_CHANGE_MODE = 9,
    /// Client is asking for shutdown the server
    CTS_STATUS_SHUTDOWN = 10,
    /// Client is asking for a cycle watering
    CTS_FORCE_CYCLE = 11,
    /// Client is asking for a sector watering
    CTS_FORCE_SECTOR = 12,
    /// Client is asking for full data
    CTS_GET_FULLDB = 13,
    /// Client is supplying some db update
    CTS_SYNC_DB = 14,
    /// Client is asking for weather history
    CTS_GET_WEATHER_HIST = 15,
    /// To signal do the server that we have a client connected
    CLIENT_CONNECTION = 16,
    /// To signal do the clients that the server is connected
    #[default]
    SERVER_CONNECTION = 17,
    // Example for future use when we have sensors/devices to connect thru mqtt
    // DEVICE_1_CONNECTION = 18,
    // /// Though it might be needed ... after 2 years devolopping still no use case for this
    // NULL = 19,
    SHELLIES = 18,
}

pub const COUNT_TOPIC: usize = 19;

pub const TOPIC: &[&str; COUNT_TOPIC] = &[
    "LAGARTO_CONTROLLER/STC/DATA/WEATHER",
    "LAGARTO_CONTROLLER/STC/DATA/SYNCDB",
    "LAGARTO_CONTROLLER/STC/DATA/FULLDB",
    "LAGARTO_CONTROLLER/STC/DATA/ALERT",
    "LAGARTO_CONTROLLER/STC/DATA/ALERT_RESET",
    "LAGARTO_CONTROLLER/STC/DATA/LOG_ERROR",
    "LAGARTO_CONTROLLER/STC/DATA/WEATHER/HIST/PUT",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR",
    "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE",
    "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR",
    "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB",
    "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB",
    "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET",
    "LAGARTO_CONTROLLER/CONNECTION/CLIENT",
    "LAGARTO_CONTROLLER/CONNECTION/SERVER",
    "SHELLIES",
];

pub const TOPIC_TEST: &[&str; COUNT_TOPIC] = &[
    "LAGARTO_CONTROLLER/STC/DATA/WEATHER/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/SYNCDB/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/FULLDB/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/ALERT/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/ALERT_RESET/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/LOG_ERROR/TEST",
    "LAGARTO_CONTROLLER/STC/DATA/WEATHER/HIST/PUT/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET/TEST",
    "LAGARTO_CONTROLLER/CONNECTION/CLIENT/TEST",
    "LAGARTO_CONTROLLER/CONNECTION/SERVER/TEST",
    "SHELLIES",
];

pub const SUBS_COUNT: usize = 11;

pub const SUBS: &[&str; SUBS_COUNT] = &[
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE",
    "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE",
    "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown",
    "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB",
    "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB",
    "LAGARTO_CONTROLLER/CONNECTION/CLIENT",
    "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET",
    "SHELLIES/#",
];

pub const SUBS_TEST: &[&str; SUBS_COUNT] = &[
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE/TEST",
    "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB/TEST",
    "LAGARTO_CONTROLLER/CONNECTION/CLIENT/TEST",
    "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET/TEST",
    "SHELLIES/#",
];

pub const QOS_TOPICS: &[i32; SUBS_COUNT] = &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

impl fmt::Display for Topic {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self != Topic::SHELLIES {
            let suffix = if unsafe { TESTING } { "/TEST" } else { "" };
            write!(f, "{}{}", TOPIC[*self as usize], suffix)
        } else {
            write!(f, "{}", TOPIC[*self as usize])
        }
    }
}

const REF_STR_INI: usize = 19;
impl Topic {
    #[inline]
    pub fn from_string(topic: &str) -> Topic {
        if topic.contains("shellies") {
            Topic::SHELLIES
        } else if topic.len() >= REF_STR_INI {
            if !unsafe { TESTING } {
                // using a slice after char 19, trying to compare less bytes
                match &topic[REF_STR_INI..] {
                    "STC/DATA/WEATHER" => Topic::STC_WEATHER,
                    "STC/DATA/SYNCDB" => Topic::STC_SYNC_DB,
                    "STC/DATA/FULLDB" => Topic::STC_SND_FULLDB,
                    "STC/DATA/ALERT" => Topic::STC_SND_ALERT,
                    "STC/DATA/ALERT_RESET" => Topic::STC_SND_ALERT_RESET,
                    "STC/DATA/LOG_ERROR" => Topic::STC_SND_LOG_ERROR,
                    "STC/DATA/WEATHER/HIST/PUT" => Topic::STC_WEATHER_HIST,
                    "CTS/STATUS/STOP_CYCLE" => Topic::CTS_STOP_CYCLE,
                    "CTS/STATUS/STOP_SECTOR" => Topic::CTS_STOP_SECTOR,
                    "CTS/STATUS/CHANGE_MODE" => Topic::CTS_STATUS_CHANGE_MODE,
                    "CTS/STATUS/ShutDown" => Topic::CTS_STATUS_SHUTDOWN,
                    "CTS/STATUS/FORCE_CYCLE" => Topic::CTS_FORCE_CYCLE,
                    "CTS/STATUS/FORCE_SECTOR" => Topic::CTS_FORCE_SECTOR,
                    "CTS/DATA/GET_FULLDB" => Topic::CTS_GET_FULLDB,
                    "CTS/DATA/SYNCDB" => Topic::CTS_SYNC_DB,
                    "CTS/DATA/WEATHER/HIST/GET" => Topic::CTS_GET_WEATHER_HIST,
                    "CONNECTION/CLIENT" => Topic::CLIENT_CONNECTION,
                    "CONNECTION/SERVER" => Topic::SERVER_CONNECTION,
                    // "CONNECTION/DEVICE1" => Topic::DEVICE_1_CONNECTION,
                    _ => Topic::SERVER_CONNECTION,
                }
            } else {
                match &topic[REF_STR_INI..] {
                    "STC/DATA/WEATHER/TEST" => Topic::STC_WEATHER,
                    "STC/DATA/SYNCDB/TEST" => Topic::STC_SYNC_DB,
                    "STC/DATA/GET_FULLDB/TEST" => Topic::STC_SND_FULLDB,
                    "STC/DATA/ALERT/TEST" => Topic::STC_SND_ALERT,
                    "STC/DATA/ALERT_RESET/TEST" => Topic::STC_SND_ALERT_RESET,
                    "STC/DATA/LOG_ERROR/TEST" => Topic::STC_SND_LOG_ERROR,
                    "STC/DATA/WEATHER/HIST/PUT/TEST" => Topic::STC_WEATHER_HIST,
                    "CTS/STATUS/STOP_CYCLE/TEST" => Topic::CTS_STOP_CYCLE,
                    "CTS/STATUS/STOP_SECTOR/TEST" => Topic::CTS_STOP_SECTOR,
                    "CTS/STATUS/CHANGE_MODE/TEST" => Topic::CTS_STATUS_CHANGE_MODE,
                    "CTS/STATUS/ShutDown/TEST" => Topic::CTS_STATUS_SHUTDOWN,
                    "CTS/STATUS/FORCE_CYCLE/TEST" => Topic::CTS_FORCE_CYCLE,
                    "CTS/STATUS/FORCE_SECTOR/TEST" => Topic::CTS_FORCE_SECTOR,
                    "CTS/DATA/GET_FULLDB/TEST" => Topic::CTS_GET_FULLDB,
                    "CTS/DATA/SYNCDB/TEST" => Topic::CTS_SYNC_DB,
                    "CTS/DATA/WEATHER/HIST/GET/TEST" => Topic::CTS_GET_WEATHER_HIST,
                    "CONNECTION/CLIENT/TEST" => Topic::CLIENT_CONNECTION,
                    "CONNECTION/SERVER/TEST" => Topic::SERVER_CONNECTION,
                    // "CONNECTION/DEVICE1/TEST" => Topic::DEVICE_1_CONNECTION,
                    _ => Topic::SERVER_CONNECTION,
                }
            }
        } else {
            Topic::SERVER_CONNECTION
        }
    }
}

