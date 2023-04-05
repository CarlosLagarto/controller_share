use fastuuid::Generator;
use serde::{Deserialize, Serialize};

use crate::app_time::ctrl_time::*;
use crate::data_structs::client::db_sync::*;
use crate::data_structs::msgs::{alert::*, connection::*, log_error::*, topic::*, weather::*};
use crate::data_structs::rega::{mode::*, running_ptr::*};
use crate::lib_serde::*;
use crate::services::weather::weather_history::*;
use ctrl_prelude::domain_types::*;

pub const CONTROLLER_CLIENT_ID: &str = "controlador";

/// DESIGN NOTE
/// To add one variable 
/// - add the field in .file context_control.json
/// - add the field in the struct build defaults
///
/// Dimension 112
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    #[serde(skip)]
    pub topic: Topic,
    pub client_id: String, 
    pub time: UTC_UNIX_TIME,
    pub uuid: Option<String>, 
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModeMsg {
    pub header: Option<Header>,
    pub mode: Mode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherHstryMsg {
    pub header: Option<Header>,
}

impl WeatherHstryMsg{
    #[inline]
    pub fn uuid(&self) -> String {
        let mut res = "".to_owned();
        if let Some(h) = &self.header {
            if let Some(uuid) = &h.uuid {
                res = uuid.clone();
            }
        }
        res
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleMsg {
    pub header: Option<Header>,
    pub cycle_id: CYCLE_ID,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorMsg {
    pub header: Option<Header>,
    pub running_ptr: RunningPtr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBSyncMsg {
    pub header: Option<Header>,
    pub db_sync: Box<DBSync>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetFullDB {
    pub header: Option<Header>,
}

/// Dimension = 96
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtMsgIn {
    Connection(Connection),
    ChangeMode(ModeMsg),
    WeatherHistory(WeatherHstryMsg),
    Cycle(CycleMsg),
    Sector(SectorMsg),
    DBSync(Box<DBSyncMsg>),
    GetFullDB(GetFullDB),
    Alert(Alert),
    ShutDown(Header),
}

/// Dimension = 96
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtMsgOut {
    Connection(Connection),
    LogError(LogError),
    ChangeMode(ModeMsg),
    WeatherHistory(Box<WeatherHstry>),
    DBSync(Box<DBSyncMsg>),
    Alert(Alert),
    Weather(Box<Weather>),
}

impl ExtMsgOut {
    #[rustfmt::skip]
    #[inline]
    pub fn new(topic: Topic, msg: ExtMsgOut, time: CtrlTime) -> ExtMsgOut {
        let generator = Generator::new();
        let mut buffer: [u8; 36] = [0; 36];
        let uuid: String;

        unsafe { uuid = generator.hex128_as_str_unchecked(&mut buffer).to_string(); }

        let header = Header {
            topic,
            client_id: CONTROLLER_CLIENT_ID.to_string(),
            uuid: Some(uuid),
            time: time.ux_ts(),
        };

        match msg {
            ExtMsgOut::Connection(mut connection) => {
                connection.header = Some(header);
                ExtMsgOut::Connection(connection)
            },
            ExtMsgOut::LogError(mut log_error) => {
                log_error.header = Some(header);
                ExtMsgOut::LogError(log_error)
            },
            ExtMsgOut::ChangeMode(mut mode) => {
                mode.header = Some(header);
                ExtMsgOut::ChangeMode(mode)
            },
            ExtMsgOut::WeatherHistory(mut weather_history) => {
                weather_history.header = Some(header);
                ExtMsgOut::WeatherHistory(weather_history)
            },
            ExtMsgOut::DBSync(mut db_sync) => {
                db_sync.header = Some(header);
                ExtMsgOut::DBSync(db_sync)
            },
            ExtMsgOut::Alert(mut alert) => {
                alert.header = Some(header);
                ExtMsgOut::Alert(alert)
            },
            ExtMsgOut::Weather(mut weather) => {
                weather.header = Some(header);
                ExtMsgOut::Weather(weather)
            },
        }
    }

    #[inline]
    pub fn header(&self) -> Option<&Header> {
        match self {
            ExtMsgOut::Connection(connection) => connection.header.as_ref(),
            ExtMsgOut::ChangeMode(mode) => mode.header.as_ref(),
            ExtMsgOut::WeatherHistory(weather_history) => weather_history.header.as_ref(),
            ExtMsgOut::DBSync(db_sync) => db_sync.header.as_ref(),
            ExtMsgOut::Alert(alert) => alert.header.as_ref(),
            ExtMsgOut::LogError(log_error) => log_error.header.as_ref(),
            ExtMsgOut::Weather(weather) => weather.header.as_ref(),
        }
    }
}

impl ExtMsgIn {
    #[inline]
    pub fn new(topic: Topic, payload: &str) -> JsonResult<ExtMsgIn> {
        match data_from_str::<ExtMsgIn>(payload) {
            Ok(mut msg) => {
                msg.set_topic(topic);
                Ok(msg)
            }
            Err(err) => Err(ConversionError::DeserializationIssue(payload.to_owned(), err.to_string())),
        }
    }

    #[inline]
    pub fn header(&self) -> Option<&Header> {
        match self {
            ExtMsgIn::Connection(connection) => connection.header.as_ref(),
            ExtMsgIn::ChangeMode(mode) => mode.header.as_ref(),
            ExtMsgIn::WeatherHistory(weather_history) => weather_history.header.as_ref(),
            ExtMsgIn::DBSync(db_sync) => db_sync.header.as_ref(),
            ExtMsgIn::GetFullDB(get_ful_db) => get_ful_db.header.as_ref(),
            ExtMsgIn::Alert(alert) => alert.header.as_ref(),
            ExtMsgIn::Cycle(cycle_msg) => cycle_msg.header.as_ref(),
            ExtMsgIn::Sector(sector_msg) => sector_msg.header.as_ref(),
            ExtMsgIn::ShutDown(header) => Some(header),
        }
    }

    #[inline]
    pub fn set_topic(&mut self, topic: Topic) {
        match self {
            ExtMsgIn::Connection(connection) => connection.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::ChangeMode(mode) => mode.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::WeatherHistory(weather_history) => weather_history.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::DBSync(db_sync) => db_sync.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::GetFullDB(get_full_db) => get_full_db.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::Alert(alert) => alert.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::Cycle(cycle_msg) => cycle_msg.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::Sector(sector_msg) => sector_msg.header.as_mut().unwrap().topic = topic,
            ExtMsgIn::ShutDown(header) => header.topic = topic,
        }
    }
}

impl Json for ExtMsgIn {
    #[inline]
    #[rustfmt::skip]
    fn json(&self) -> JsonResult<String> { data_to_str(&self) }
}

impl Json for ExtMsgOut {
    #[inline]
    #[rustfmt::skip]
    fn json(&self) -> JsonResult<String> { data_to_str(&self) }
}
