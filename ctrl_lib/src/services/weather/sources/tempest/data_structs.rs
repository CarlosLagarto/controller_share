#![allow(clippy::derive_partial_eq_without_eq)]
use ctrl_prelude::domain_types::*;
use serde::{Deserialize, Serialize};

//     token para lagarto:  "47209cc1-a347-432e-ad16-d0cd0cc3ff2e"
//     serial number:       "ST-00085776" ?
//     station id:          79881
//     device_id:           211801
//
// GET STATION META DATA
// Retrieve a list of your stations along with all connected devices.
// https://swd.weatherflow.com/swd/rest/stations?token=[your_access_token]
// https://swd.weatherflow.com/swd/rest/stations?token=47209cc1-a347-432e-ad16-d0cd0cc3ff2e

// Este parece ser o que têm mais info
// GET LATEST STATION OBSERVATION
// Get the latest most recent observation for your station.
// https://swd.weatherflow.com/swd/rest/observations/station/[your_station_id]?token=[your_access_token]
// https://swd.weatherflow.com/swd/rest/observations/station/79881?token=47209cc1-a347-432e-ad16-d0cd0cc3ff2e

// GET LATEST DEVICE OBSERVATION
// Get the latest observation from one of your devices.
// https://swd.weatherflow.com/swd/rest/observations/?device_id=[your_device_id]&token=[your_access_token]
// https://swd.weatherflow.com/swd/rest/observations/?device_id=211801&token=47209cc1-a347-432e-ad16-d0cd0cc3ff2e

// GET FORECAST
// https://swd.weatherflow.com/swd/rest/better_forecast/?device_id=211801&token=47209cc1-a347-432e-ad16-d0cd0cc3ff2e
// este deve ser um serviço premium que não está autorizado via rest

// Esta info está aqui para referência, mas decidi não usar web sockets.  Vou fazer pull e não push.  Só porque sim.  É um pouco mais simples, e já tenho a lógica montada.
// OPEN A WEBSOCKET CONNECTION
// wss://ws.weatherflow.com/swd/data?token=[your_access_token]
// LISTEN FOR OBSERVATIONS
// Send a JSON message over the websocket connection to start listening for observations from the device. After sending this message your connected websocket client should receive a new observation JSON message every minute.
// {
// 	"type":"listen_start",
// 	"device_id": [your_device_id],
// 	"id":"random-id-12345"
// }
// ## References
// [`WeatherFlow UDP`](https://weatherflow.github.io/Tempest/api/udp/v171/)

pub const BUF_SIZE: usize = 400;

/// Top level abstraction using serde tag feature to select enum variant based on the value of the JSON `type` field.
///
/// The variant names directly map to the type names with `snake_case` conversion.
///
/// Dimension 144
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tempest {
    /// Rain Start Event [type = evt_precip]
    EvtPrecip(EvtPrecip),
    /// Lightning Strike Event [type = evt_strike]
    EvtStrike(EvtStrike),
    /// Rapid Wind [type = rapid_wind]
    RapidWind(RapidWind),
    /// Observation (AIR) [type = obs_air]
    ObsAir(ObsAir),
    /// Observation (Sky) [type = obs_sky]
    ObsSky(ObsSky),
    /// Observation (Tempest) [type = obs_st]
    ObsSt(ObsSt),
    /// Status (device) [type = device_status]
    DeviceStatus(DeviceStatus),
    /// Status (hub) [type = hub_status]
    HubStatus(HubStatus),
}

/// Structure defining the [Rain Start Event] enum variant.
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct EvtPrecip {
    pub serial_number: String, // SK-00008453
    pub hub_sn: String,        // HB-0000001
    pub evt: EvtPrecipEvt,     // [1493322445]
}

/// Structure defining the [Lightning Strike] enum variant.
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct EvtStrike {
    pub serial_number: String, // SK-00008453
    pub hub_sn: String,        // HB-0000001
    pub evt: EvtStrikeEvt,     // [1493322445,27,3848]
}

/// Structure defining the [Rapid Wind] enum variant.
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RapidWind {
    pub serial_number: String, // "ST-00028405"
    pub hub_sn: String,        // "HB-00027548"
    pub ob: RapidWindOb,       // [1635567982,1.15,6]
}

/// Structure defining the [Air Observation] enum variant.
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsAir {
    pub serial_number: String, // "ST-00028405"
    pub hub_sn: String,        // "HB-00027548"
    pub obs: Vec<ObsAirObs>,   // [[1493164835,835.0,10.0,45,0,0,3.46,1]]
    pub firmware_revision: u8, // 17
}

/// Structure defining the [Sky Observation] enum variant
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsSky {
    pub serial_number: String, // "SK-00008453"
    pub hub_sn: String,        // "HB-00000001"
    pub obs: Vec<ObsSkyObs>,   // [[1493321340,9000,10,0.0,2.6,4.6,7.4,187,3.12,1,130,null,0,3]]
    pub firmware_revision: u8, // 29
}

/// Structure defining the [Tempest Observation] enum variant
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ObsSt {
    pub serial_number: String,  // "SK-00000512"
    pub hub_sn: String,         // "HB-00013030"
    pub obs: Vec<ObsStObs>,     // [[1588948614,0.18,0.22,0.27,144,6,1017.57,22.37,50.26,328,0.03,3,0.00000,0,0,0,2.410,1]]
    pub firmware_revision: u32, // 129
}

/// Structure defining the [Device Status] enum variant
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatus {
    pub serial_number: String,  // "AR-00004049"
    pub hub_sn: String,         // "HB-00000001"
    pub timestamp: UTC_UNIX_TIME,         // 1510855923
    pub uptime: u32,            // 2189
    pub voltage: f32,           // 3.50
    pub firmware_revision: u32, // 17
    pub rssi: i32,              // -17
    pub hub_rssi: i32,          // -87
    pub sensor_status: u32,     // 0
    pub debug: u32,             // 0
}

/// Structure defining the [Hub Status] enum variant.
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct HubStatus {
    pub serial_number: String,     // "HB-00027548"
    pub firmware_revision: String, // 171
    pub uptime: u32,               // 86271
    pub rssi: i32,                 // -29
    pub timestamp: UTC_UNIX_TIME,            // 1639424393
    pub reset_flags: String,       // "BOR,PIN,POR"
    pub seq: u32,                  // 8508
    // pub fs: Vec<u32>,              // [1,0,15675411,524288] -- internal use
    pub radio_stats: RadioStats, // [25,1,0,3,17773]
    pub mqtt_stats: Vec<u32>,    // [20,0] -- internal use
}

/// Precipitation event detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct EvtPrecipEvt {
    pub epoch: UTC_UNIX_TIME, // 1635567982 Seconds
}

/// Lightning strike event detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct EvtStrikeEvt {
    pub epoch: UTC_UNIX_TIME,    // 1635567982 Seconds
    pub distance: u16, // km
    pub energy: u16,
}

/// Rapid Wind event detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct RapidWindOb {
    pub epoch: UTC_UNIX_TIME,          // 1635567982 Seconds
    pub wind_speed: f32,     // 1.15 mps
    pub wind_direction: u32, // 6 Degrees
}

/// Air Observation detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct ObsAirObs {
    pub epoch: UTC_UNIX_TIME,                         // 1635567982 Seconds
    pub station_pressure: f32,              // 835.0 MB
    pub air_temperature: f32,               // 10.0 Degrees C
    pub relative_humidity: u32,             // 45 %
    pub lightning_strike_count: u32,        // 0 Km
    pub lightning_strike_avg_distance: u32, // 0 Km
    pub battery: f32,
    pub report_interval: u32, // 1 Minutes
}

/// Sky Observation detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct ObsSkyObs {
    pub epoch: UTC_UNIX_TIME,          // 1635567982 Seconds
    pub illuminance: u32,    // 835.0 MB
    pub uv: u32,             // 10.0 Degrees C
    pub rain_minute: f32,    // 45 %
    pub wind_lull_min3: f32, // 0 Km
    pub wind_avg: f32,       // 0 Km
    pub wind_gust_max3: f32, // 0 Km
    pub wind_direction: u32, // 0 Km
    pub battery: f32,
    pub report_interval: u32, // 1 Minutes
    pub solar_radiation: u32,
    pub rain_day: Option<u32>,
    pub precipitation_type: u8, // 0 = none, 1 = rain, 2 = hail
    pub wind_sample_interval: u32,
}

/// Tempest Observation detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct ObsStObs {
    pub epoch: UTC_UNIX_TIME,          // 1635567982 Seconds
    pub wind_lull_min3: f32, // 0 Km
    pub wind_avg: f32,       // 0 Km
    pub wind_gust_max3: f32, // 0 Km
    pub wind_direction: u32, // 0 Km
    pub wind_sample_interval: u32,
    pub station_pressure: f32,
    pub air_temperature: f32, // degrees C
    pub relative_humidity: f32,
    pub illuminance: u32, // 835.0 MB
    pub uv: f32,          // 10.0 Degrees C
    pub solar_radiation: u32,
    pub rain_minute: f32,       // 45 %
    pub precipitation_type: u8, // 0 = none, 1 = rain, 2 = hail, 3 = rain + hail
    pub lightning_strike_dist: u32,
    pub lightning_strike_count: u32,
    pub battery: f32,
    pub report_interval: u32, // 1 Minutes
}

/// Radio Stats detail.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct RadioStats {
    pub version: u32,     // Version [25]
    pub reboots: u32,     // Reboot Count [1]
    pub i2c_errors: u32,  // I2C Bus Error Counts [0]
    pub radio_status: u8, // Radio Status (0 = Radio Off, ...)
    pub network_id: u32,  // Radio Network ID [2839]
}
