#![allow(unused_imports)]
#![allow(dead_code)]
use std::collections::HashMap;

use ctrl_prelude::domain_types::*;
use serde::Deserialize;

use crate::services::electronics::actuator::*;

#[derive(Debug, Deserialize)]
pub struct Device {
    #[serde(rename = "type")]
    type_: String,
    mac: String,
    hostname: String,
    num_outputs: u8,
}

#[derive(Debug, Deserialize)]
pub struct WifiAp {
    enabled: bool,
    ssid: String,
    key: String,
}

#[derive(Debug, Deserialize)]
pub struct WifiStaSettings {
    enabled: bool,
    ssid: String,
    ipv4_method: String,
    ip: String,
    gw: String,
    mask: String,
    dns: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WifiSta1 {
    enabled: bool,
    ssid: Option<String>,
    ipv4_method: String,
    ip: Option<String>,
    gw: Option<String>,
    mask: Option<String>,
    dns: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApRoaming {
    enabled: bool,
    threshold: i16,
}

#[derive(Debug, Deserialize)]
pub struct MqttSettings {
    enable: bool,
    server: String,
    user: String,
    id: String,
    reconnect_timeout_max: f32,
    reconnect_timeout_min: f32,
    clean_session: bool,
    keep_alive: u16,
    max_qos: u8,
    retain: bool,
    update_period: u16,
}

#[derive(Debug, Deserialize)]
pub struct COiot {
    enabled: bool,
    update_period: u16,
    peer: String,
}

#[derive(Debug, Deserialize)]
pub struct SNTP {
    server: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    enabled: bool,
    unprotected: bool,
    username: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildInfo {
    build_id: String,
    build_timestamp: String,
    build_version: String,
}

#[derive(Debug, Deserialize)]
pub struct Cloud {
    enabled: bool,
    connected: bool,
}

#[derive(Debug, Deserialize)]
pub struct Switch {
    relay_num: i16,
}

#[derive(Debug, Deserialize)]
pub struct Actions {
    active: bool,
    names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct HWInfo {
    hw_revision: String,
    batch_id: u16,
}

#[derive(Debug, Deserialize)]
pub struct Relay {
    name: Option<String>,
    appliance_type: String,
    ison: bool,
    has_timer: bool,
    default_state: String,
    btn_type: String,
    btn_reverse: u8,
    auto_on: f32,
    auto_off: f32,
    power: f32,
    schedule: bool,
    schedule_rules: Vec<String>, //[]     REVIEW STILL TO UNDERSTAND THIS TYPE
}

#[derive(Debug, Deserialize)]
pub struct Sensor {} //REVIEW STILL TO FIND OUT WHAT IS INSIDE THIS

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct Temperature{
    tC: f32,
    tF: f32,
}

#[derive(Debug, Deserialize)]
pub struct Humidity {} //REVIEW STILL TO FIND OUT WHAT IS INSIDE THIS

#[derive(Debug, Deserialize)]
pub struct Settings {
    device: Device,                   // {"device":{"type":"SHSW-1","mac":"485519C96846","hostname":"shelly1-485519C96846","num_outputs":1},
    wifi_ap: WifiAp,                  // "wifi_ap":{"enabled":false,"ssid":"shelly1-485519C96846","key":""},
    wifi_sta: WifiStaSettings, // "wifi_sta":{"enabled":true,"ssid":"lagarto-lisboa2.4","ipv4_method":"static","ip":"192.168.1.67","gw":"192.168.1.1","mask":"255.255.255.0","dns":null},
    wifi_sta1: WifiSta1,       // "wifi_sta1":{"enabled":false,"ssid":null,"ipv4_method":"dhcp","ip":null,"gw":null,"mask":null,"dns":null},
    ap_roaming: ApRoaming,     // "ap_roaming":{"enabled":false,"threshold":-70},
    mqtt: MqttSettings, // "mqtt": {"enable":true,"server":"192.168.1.64:1883","user":"","id":"shelly1-485519C96846","reconnect_timeout_max":60.000000,"reconnect_timeout_min":2.000000,"clean_session":true,"keep_alive":60,"max_qos":1,"retain":false,"update_period":30},
    coiot: COiot,       // "coiot": {"enabled":true,"update_period":15,"peer":""},
    sntp: SNTP,         // "sntp":{"server":"time.google.com","enabled":true},
    login: Login,       // "login":{"enabled":false,"unprotected":false,"username":"admin"},
    pin_code: String,   // "pin_code":"",
    name: String,       // "name":"Relay 1",
    fw: String,         // "fw":"20221027-091427/v1.12.1-ga9117d3",
    factory_reset_from_switch: bool, // "factory_reset_from_switch":true,
    discoverable: bool, // "discoverable":true,
    build_info: BuildInfo, // "build_info":{"build_id":"20221027-091427/v1.12.1-ga9117d3","build_timestamp":"2022-10-27T09:14:27Z","build_version":"1.0"},
    cloud: Cloud,       // "cloud":{"enabled":false,"connected":false},
    timezone: String,   // "timezone":"Europe/Lisbon",
    lat: f32,           // "lat":38.759960,
    lng: f32,           // "lng":-9.157770,
    tzautodetect: bool, // "tzautodetect":true,
    tz_utc_offset: u8,  // "tz_utc_offset":0,
    tz_dst: bool,       // "tz_dst":false,
    tz_dst_auto: bool,  // "tz_dst_auto":true,
    time: String,       // "time":"08:32",
    unixtime: UTC_UNIX_TIME, // "unixtime":1669883564,
    debug_enable: bool, // "debug_enable":false,
    allow_cross_origin: bool, // "allow_cross_origin":false,
    ext_switch_enable: bool, // "ext_switch_enable":false,
    ext_switch_reverse: bool, // "ext_switch_reverse":false,
    ext_switch: HashMap<u16, Switch>, // "ext_switch":{"0":{"relay_num":-1}},
    actions: Actions, // "actions":{"active":false, "names":["btn_on_url","btn_off_url","longpush_url","shortpush_url","out_on_url","out_off_url","lp_on_url","lp_off_url","report_url","report_url","report_url","ext_temp_over_url","ext_temp_under_url","ext_temp_over_url","ext_temp_under_url","ext_temp_over_url","ext_temp_under_url","ext_hum_over_url","ext_hum_under_url"]},
    hwinfo: HWInfo,   // "hwinfo":{"hw_revision":"prod-191217", "batch_id":1},
    mode: String,     // "mode" :"relay",
    longpush_time: u16, // "longpush_time":800,
    relays: Vec<Relay>, // "relays":[{"name":null,"appliance_type":"General","ison":true,"has_timer":false,"default_state":"off","btn_type":"toggle","btn_reverse":0,"auto_on":0.00,"auto_off":0.00,"power":0.00,"schedule":false,"schedule_rules":[]}],
    ext_sensors: Sensor, // "ext_sensors":{},
    ext_temperature: Temperature, // "ext_temperature":{},
    ext_humidity: Humidity, // "ext_humidity":{},
    eco_mode_enabled: bool, // "eco_mode_enabled":true}
}

#[derive(Debug, Deserialize)]
pub struct ActionStats {
    skipped: u16,
}

#[derive(Debug, Deserialize)]
pub struct RelayStatus {
    pub ison: bool,
    pub has_timer: bool,
    pub timer_started: UTC_UNIX_TIME,
    pub timer_duration: u64,
    pub timer_remaining: u64,
    pub source: String,
}

impl RelayStatus {
    #[inline]
    pub fn is_cmd_response_ok(&self, cmd: ActuatorCommand) -> bool {
        match cmd {
            ActuatorCommand::On => self.ison,
            ActuatorCommand::Off => !self.ison,
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Meter {
    power: f32,
    is_valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct Input {
    input: u16,
    event: String,
    event_cnt: u16,
}

#[derive(Debug, Deserialize)]
pub struct Update {
    status: String,
    has_update: bool,
    new_version: String,
    old_version: String,
}

#[derive(Debug, Deserialize)]
pub struct MqttStatus {
    connected: bool,
}

#[derive(Debug, Deserialize)]
pub struct WifiStaStatus {
    connected: bool,
    ssid: Option<String>,
    ip: Option<String>,
    rssi: i16,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub wifi_sta: WifiStaStatus,      // {"wifi_sta":{"connected":true,"ssid":"lagarto-lisboa2.4","ip":"192.168.1.67","rssi":-76},
    pub cloud: Cloud,                 // "cloud":{"enabled":false,"connected":false},
    pub mqtt: MqttStatus,             //"mqtt":{"connected":true},
    pub time: String,                 // "time":"11:03",
    pub unixtime: UTC_UNIX_TIME,      //"unixtime":1669892597,
    pub serial: u16,                  //"serial":14,
    pub has_update: bool,             //"has_update":false,
    pub mac: String,                  //"mac":"485519C96846",
    pub cfg_changed_cnt: u16,         //"cfg_changed_cnt":4,
    pub actions_stats: ActionStats,   //"actions_stats":{"skipped":0},
    pub relays: Vec<RelayStatus>,     //"relays":[{"ison":true,"has_timer":false,"timer_started":0,"timer_duration":0,"timer_remaining":0,"source":"http"}],
    pub meters: Vec<Meter>,           //"meters":[{"power":0.00,"is_valid":true}],
    pub inputs: Vec<Input>,           //"inputs":[{"input":0,"event":"","event_cnt":0}],
    pub ext_sensors: Sensor,          //"ext_sensors":{},
    pub ext_temperature: Temperature, //"ext_temperature":{},
    pub ext_humidity: Humidity,       //"ext_humidity":{},
    pub update: Update, //"update":{"status":"idle","has_update":false,"new_version":"20221027-091427/v1.12.1-ga9117d3","old_version":"20221027-091427/v1.12.1-ga9117d3"},
    pub ram_total: u64, //"ram_total":51688,
    pub ram_free: u64,  //"ram_free":39764,
    pub fs_size: u64,   //"fs_size":233681,
    pub fs_free: u64,   //"fs_free":150600,
    pub uptime: u64,    //"uptime":239560}
}

#[derive(Debug, Deserialize)]
pub struct Shelly {
    #[serde(rename = "type")]
    type_: String,
    mac: String,
    auth: bool,
    fw: String,
    discoverable: bool,
    longid: u64,
    num_outputs: u16,
}

#[derive(Debug, Deserialize)]
pub struct Roller2PMStatus{
    // {"state":"stop", "source":"limit_switch","power":0.0,"is_valid":true,"safety_switch":false, "overtemperature":false, "stop_reason":"normal", 
    //"last_direction":"close","calibrating":false, "positioning":false}
    state: String,
    source: String,
    power: f32,
    is_valid: bool,
    safety_switch: bool,
    overtemperature: bool,
    stop_reason: String,
    last_direction: String,
    calibrating: bool,
    positioning: bool
}

impl Roller2PMStatus {
    #[inline]
    pub fn is_cmd_response_ok(&self, cmd: ActuatorCommand) -> bool {
        match cmd {
            // ActuatorCommand::On => self.ison,
            // ActuatorCommand::Off => !self.ison,
            ActuatorCommand::Up=> self.is_valid,
            ActuatorCommand::Stop=> self.is_valid,
            ActuatorCommand::Down=> self.is_valid,
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AEnergy{
    by_minute: Vec<f32>,
    minute_ts: UTC_UNIX_TIME,
    total: f32,
}

#[derive(Debug, Deserialize)]
pub struct Cover{
    id: u8,
    aenergy: AEnergy,
}

#[derive(Debug, Deserialize)]
pub struct ShellyPlus2pmEventNotifyStatusParams{
    ts: f32,
    #[serde(rename(deserialize = "cover:0"))]
    cover: Cover,
}

#[derive(Debug, Deserialize)]
pub struct ShellyPlus2pmRPCEvent{
    src: String,
    dst: String,
    method: String,
    params: ShellyPlus2pmEventNotifyStatusParams,
}

#[derive(Debug, Deserialize)]
pub struct ShellyPlus2pmStatus{
    id: usize,
    source: String,
    state: String,
    apower: f32,
    voltage: f32,
    current:f32,
    pf:f32,
    aenergy:AEnergy,
    temperature: Temperature,
    pos_control: bool,
}

pub mod test {
    use crate::{lib_serde::data_from_str, services::electronics::model::shelly_structs::*};

    #[test]
    fn test_2pm_mqtt_status2(){
        let in_msg = r#"{"id":0, "source":"init", "state":"stopped","apower":0.0,"voltage":236.3,"current":0.000,"pf":0.00,"aenergy":{"total":0.000,"by_minute":[0.000,0.000,0.000],"minute_ts":1678018619},"temperature":{"tC":48.0, "tF":118.4},"pos_control":false}"#;
        let msg = data_from_str::<ShellyPlus2pmStatus>(&in_msg);
        println!("{:?}", msg);
    }

    #[test]
    fn test_2pm_mqtt_status1(){
        let in_msg = r#"{"src":"shellyplus2pm-90380c35a854","dst":"shellies/shellyplus2pm-90380c35a854/events","method":"NotifyStatus","params":{"ts":1678018620.04,"cover:0":{"id":0,"aenergy":{"by_minute":[0.000,0.000,0.000],"minute_ts":1678018619,"total":0.000}}}}"#;
        let msg = data_from_str::<ShellyPlus2pmRPCEvent>(&in_msg);
        println!("{:?}", msg);
    }

    #[test]
    fn test_roller_2pm_status() {
        let in_msg = r#"{"state":"stop", "source":"limit_switch","power":0.0,"is_valid":true,"safety_switch":false, "overtemperature":false, "stop_reason":"normal","last_direction":"close","calibrating":false, "positioning":false}"#;
        let msg = data_from_str::<Roller2PMStatus>(&in_msg);
        println!("{:?}", msg);
    }

    #[test]
    fn test_cmd_on_off() {
        let in_msg = r#"{"ison":false,"has_timer":false,"timer_started":0,"timer_duration":0,"timer_remaining":0,"source":"http"}"#;
        let msg = data_from_str::<RelayStatus>(&in_msg);
        println!("{:?}", msg);
    }

    #[test]
    fn test_shelly() {
        let in_msg =
            r#"{"type":"SHSW-1","mac":"485519C96846","auth":false,"fw":"20221027-091427/v1.12.1-ga9117d3","discoverable":true,"longid":1,"num_outputs":1}"#;

        let msg = data_from_str::<Shelly>(&in_msg);
        println!("{:?}", msg);
    }

    #[test]
    fn test_status() {
        let in_msg = r#"{"wifi_sta":{"connected":true,"ssid":"lagarto-lisboa2.4","ip":"192.168.1.67","rssi":-76},
                                "cloud":{"enabled":false,"connected":false},
                                "mqtt":{"connected":true},
                                "time":"11:03",
                                "unixtime":1669892597,
                                "serial":14,
                                "has_update":false,
                                "mac":"485519C96846",
                                "cfg_changed_cnt":4,
                                "actions_stats":{"skipped":0},
                                "relays":[{"ison":true,"has_timer":false,"timer_started":0,"timer_duration":0,"timer_remaining":0,"source":"http"}],
                                "meters":[{"power":0.00,"is_valid":true}],
                                "inputs":[{"input":0,"event":"","event_cnt":0}],
                                "ext_sensors":{},
                                "ext_temperature":{},
                                "ext_humidity":{},
                                "update":{"status":"idle","has_update":false,"new_version":"20221027-091427/v1.12.1-ga9117d3","old_version":"20221027-091427/v1.12.1-ga9117d3"},
                                "ram_total":51688,
                                "ram_free":39764,
                                "fs_size":233681,
                                "fs_free":150600,
                                "uptime":239560}"#;

        let msg = data_from_str::<Status>(&in_msg);
        println!("{:?}", msg);
    }
    #[test]
    fn test_settings() {
        let in_msg = r#"{"device":{"type":"SHSW-1","mac":"485519C96846","hostname":"shelly1-485519C96846","num_outputs":1},
                            "wifi_ap":{"enabled":false,"ssid":"shelly1-485519C96846","key":""},
                            "wifi_sta":{"enabled":true,"ssid":"lagarto-lisboa2.4","ipv4_method":"static","ip":"192.168.1.67","gw":"192.168.1.1","mask":"255.255.255.0","dns":null},
                            "wifi_sta1":{"enabled":false,"ssid":null,"ipv4_method":"dhcp","ip":null,"gw":null,"mask":null,"dns":null},
                            "ap_roaming":{"enabled":false,"threshold":-70},
                            "mqtt": {"enable":true,"server":"192.168.1.64:1883","user":"","id":"shelly1-485519C96846","reconnect_timeout_max":60.000000,"reconnect_timeout_min":2.000000,"clean_session":true,"keep_alive":60,"max_qos":1,"retain":false,"update_period":30},
                            "coiot": {"enabled":true,"update_period":15,"peer":""},
                            "sntp":{"server":"time.google.com","enabled":true},
                            "login":{"enabled":false,"unprotected":false,"username":"admin"},
                            "pin_code":"",
                            "name":"Relay 1",
                            "fw":"20221027-091427/v1.12.1-ga9117d3",
                            "factory_reset_from_switch":true,
                            "discoverable":true,
                            "build_info":{"build_id":"20221027-091427/v1.12.1-ga9117d3","build_timestamp":"2022-10-27T09:14:27Z","build_version":"1.0"},
                            "cloud":{"enabled":false,"connected":false},
                            "timezone":"Europe/Lisbon",
                            "lat":38.759960,
                            "lng":-9.157770,
                            "tzautodetect":true,
                            "tz_utc_offset":0,
                            "tz_dst":false,
                            "tz_dst_auto":true,
                            "time":"08:32",
                            "unixtime":1669883564,
                            "debug_enable":false,
                            "allow_cross_origin":false,
                            "ext_switch_enable":false,
                            "ext_switch_reverse":false,
                            "ext_switch":{"0":{"relay_num":-1}},
                            "actions":{"active":false,
                                        "names":["btn_on_url","btn_off_url","longpush_url","shortpush_url","out_on_url","out_off_url","lp_on_url","lp_off_url","report_url","report_url","report_url","ext_temp_over_url","ext_temp_under_url","ext_temp_over_url","ext_temp_under_url","ext_temp_over_url","ext_temp_under_url","ext_hum_over_url","ext_hum_under_url"]},
                            "hwinfo":{"hw_revision":"prod-191217", "batch_id":1},
                            "mode" :"relay",
                            "longpush_time":800,
                            "relays":[{"name":null,"appliance_type":"General","ison":true,"has_timer":false,"default_state":"off","btn_type":"toggle","btn_reverse":0,"auto_on":0.00,"auto_off":0.00,"power":0.00,"schedule":false,"schedule_rules":[]}],
                            "ext_sensors":{},
                            "ext_temperature":{},
                            "ext_humidity":{},
                            "eco_mode_enabled":true}"#;

        let msg = data_from_str::<Settings>(&in_msg);
        println!("{:?}", msg);
    }
}
