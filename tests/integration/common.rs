use std::mem::size_of;

use alloc_counter::AllocCounter;

use ctrl_lib::app_context::{db_mnt_cfg::*, start_up::*};
use ctrl_lib::app_time::{ctrl_time::*, date_time::*, schedule::*};
use ctrl_lib::config::{app_config::*, db_cfg::*, geo_pos::*, mqtt_config::*, time_cfg::*, web_cfg::*, wthr_cfg::*, wtr_cfg::*};
use ctrl_lib::controller_sync::*;
use ctrl_lib::data_structs::client::{client_ctx::*, db_sync::*};
use ctrl_lib::data_structs::concurrent_queue::*;
use ctrl_lib::data_structs::msgs::{alert::*, alert_thresholds::*, connection::*, ext_message::*, int_message::*, log_error::*, topic::*};
use ctrl_lib::data_structs::rega::{command::*, internal::*, mode::*, running_ptr::*, wizard_info::*};
use ctrl_lib::data_structs::sensor::metrics::evapo_transpiracao::*;
use ctrl_lib::data_structs::sensor::{daily_value::*, stat_metric::*};
use ctrl_lib::db::db_sql_lite::*;
use ctrl_lib::services::irrigation::{cycle::*, cycle_run::*, sector::*, sector_run::*, wtr_engine::*, wtr_history::*};
use ctrl_lib::services::mqtt::mqtt_svc::*;
use ctrl_lib::services::weather::rain_pred::data_structs::*;
use ctrl_lib::services::weather::sources::simulation::{mock_weather_data::*, svc::*};
use ctrl_lib::services::weather::sources::tempest::{data_structs::*, rest::*};
use ctrl_lib::services::weather::trend::*;
use ctrl_lib::services::weather::{history_value::*, scale::*, weather_history::*, weather_inner::*, weather_svc::*};
use ctrl_lib::services::web_svc::*;
use ctrl_lib::services::{client::clients_svc::*, db_maint::db_mnt_svc::*, mqtt::*, msg_broker::msg_brkr_svc::*};
use ctrl_lib::thread_signal::{cond_var::*, raw_event::*};
use ctrl_prelude::globals::GIGA_U;

use crate::integration::naive_bayes::code::data_structs::*; //, count_alloc}

type MyAllocator = std::alloc::System;
#[allow(non_upper_case_globals)]
const MyAllocator: MyAllocator = std::alloc::System;
#[global_allocator]
static A: AllocCounter<MyAllocator> = AllocCounter(MyAllocator);

pub fn initialize_evt_mng() {
    DB_MAINT_SIG.read().set();
    NEW_DAY_SIG.read().set();
}

pub fn make_request(url: &str) -> core::result::Result<String, minreq::Error> {
    minreq::get(url).with_timeout(20).send().map_or_else(Err, |d| d.json::<String>())
}

#[test]
pub fn bench_data_structs_size() {
    println!("Alert                         : {:>3}", size_of::<Alert>());
    println!("AlertThresholds               : {:>3}", size_of::<AlrtThresholds>());
    println!("AlertType                     : {:>3}", size_of::<AlertType>());
    println!("AppConfig                     : {:>3}", size_of::<AppCfg>());
    println!("CacheFx                       : {:>3}", size_of::<CacheFx>());
    println!("ClientContext                 : {:>3}", size_of::<ClientCtx>());
    println!("ClientService                 : {:>3}", size_of::<ClntSvc>());
    println!("Command                       : {:>3}", size_of::<Command>());
    println!("CondVar                       : {:>3}", size_of::<CondVar>());
    println!("Connection                    : {:>3}", size_of::<Connection>());
    println!("CtrlTime                      : {:>3}", size_of::<CtrlTime>());
    println!("Cycle                         : {:>3}", size_of::<Cycle>());
    println!("CycleList                     : {:>3}", size_of::<CycleList>());
    println!("CycleRun                      : {:>3}", size_of::<CycleRun>());
    println!("DataArray                     : {:>3}", size_of::<DataArray>());
    println!("DateTimeE                     : {:>3}", size_of::<DateTimeE>());
    println!("DBConfig                      : {:>3}", size_of::<DBConfig>());
    println!("DBMaintConfig                 : {:>3}", size_of::<DBMntCfg>());
    println!("DBMntSvc                      : {:>3}", size_of::<DBMntSvc>());
    println!("DBSync                        : {:>3}", size_of::<DBSync>());
    println!("Duration                      : {:>3}", size_of::<std::time::Duration>());
    println!("EtData                        : {:>3}", size_of::<EtData>());
    println!("ExtMsgIn                      : {:>3}", size_of::<ExtMsgIn>());
    println!("ExtMsgOut                     : {:>3}", size_of::<ExtMsgOut>());
    println!("GeoPos                        : {:>3}", size_of::<GeoPos>());
    println!("HistoryValue                  : {:>3}", size_of::<HistoryValue>());
    println!("InnerBroker                   : {:>3}", size_of::<InnerBroker>());
    println!("InnerPersistance              : {:>3}", size_of::<InnerPersistance>());
    println!("InternalPtr                   : {:>3}", size_of::<InternalPtr>());
    println!("IntMessage                    : {:>3}", size_of::<IntMessage>());
    println!("LogError                      : {:>3}", size_of::<LogError>());
    println!("Metric                        : {:>3}", size_of::<Metric>());
    println!("MockSimulation                : {:>3}", size_of::<MockSimulation>());
    println!("Mode                          : {:>3}", size_of::<Mode>());
    println!("MQTTConfig                    : {:>3}", size_of::<MQTTConfig>());
    println!("MQTTService                   : {:>3}", size_of::<MQTTService>());
    println!("MsgBroker                     : {:>3}", size_of::<MsgBrkr>());
    println!("MsgData                       : {:>3}", size_of::<MsgData>());
    println!("MtDeque                       : {:>3}", size_of::<MtDeque<IntMessage>>());
    println!("NBDataSet                     : {:>3}", size_of::<DataSet<MAX_DAILY_ROWS, MAX_FEATURES>>());
    println!("NBBasicStats                  : {:>3}", size_of::<BasicStats>());
    println!("NBPearsonCorrelation          : {:>3}", size_of::<PearsonCorrelation>());
    println!("NBRAIN_CLASS_KEYS             : {:>3}", size_of::<[u8; 48]>());
    println!("NBSTANDARDIZE_KEYS            : {:>3}", size_of::<[usize; 45]>());
    println!("NBModelEvaluation             : {:>3}", size_of::<ModelEvaluation>());
    println!("NBVector                      : {:>3}", size_of::<Vector<MAX_FEATURES>>());
    println!("NBCMData                      : {:>3}", size_of::<CMData>());
    println!("NBDSRow                       : {:>3}", size_of::<DSRow<MAX_FEATURES>>());
    println!("NBModel                       : {:>3}", size_of::<Model>());
    println!("NBWelfordMeanAndVar           : {:>3}", size_of::<WelfordMeanAndVar>());
    println!("NBNBGaussianPrediction        : {:>3}", size_of::<NBGaussianPrediction>());
    println!("NumberGen                     : {:>3}", size_of::<NumberGen>());
    println!("Persist                       : {:>3}", size_of::<Persist>());
    println!("LightPersist                  : {:>3}", size_of::<LightPersist>());
    println!("PtrList                       : {:>3}", size_of::<PtrList>());
    println!("RawEvent                      : {:>3}", size_of::<RawEvent>());
    println!("RunningPtr                    : {:>3}", size_of::<RunningPtr>());
    println!("Scale                         : {:>3}", size_of::<Scale>());
    println!("Schedule                      : {:>3}", size_of::<Schedule>());
    println!("SecList                       : {:>3}", size_of::<SecList>());
    println!("SecRunList                    : {:>3}", size_of::<SecRunList>());
    println!("Sector                        : {:>3}", size_of::<Sector>());
    println!("SectorRun                     : {:>3}", size_of::<SectorRun>());
    println!("SensorValue                   : {:>3}", size_of::<SensorValue>());
    println!("Simulation                    : {:>3}", size_of::<Simulation>());
    println!("SMsgBroker                    : {:>3}", size_of::<SMsgBrkr>());
    println!("StartupData                   : {:>3}", size_of::<StartupData>());
    println!("SWthrCfg                      : {:>3}", size_of::<SWthrCfg>());
    println!("Tempest                       : {:>3}", size_of::<Tempest>());
    println!("TempestRest                   : {:>3}", size_of::<TempestRest>());
    println!("TimeData                      : {:>3}", size_of::<TimeData>());
    println!("TrendA                        : {:>3}", size_of::<TrendA>());
    println!("WeatherHistory                : {:>3}", size_of::<WeatherHstry>());
    println!("WeatherInner                  : {:>3}", size_of::<WeatherInner>());
    println!("WebConfig                     : {:>3}", size_of::<WebCfg>());
    println!("WebService                    : {:>3}", size_of::<WebService>());
    println!("WizardInfo                    : {:>3}", size_of::<WizardInfo>());
    println!("WthrCfg                       : {:>3}", size_of::<WthrCfg>());
    println!("WthrSvc                       : {:>3}", size_of::<WthrSvc>());
    println!("WtrCfg                        : {:>3}", size_of::<WtrCfg>());
    println!("WtrEngine                     : {:>3}", size_of::<WtrEngine>());
    println!("WaterHstry                    : {:>3}", size_of::<WaterHstry>());
    println!("SectorHstry                   : {:>3}", size_of::<SectorHstry>());
    println!("CycleHstry                    : {:>3}", size_of::<CycleHstry>());

    println!("WtrSecList                    : {:>3}", size_of::<WtrSecList>());
}

#[test]
fn test_is_valid_condition_simulation() {
    let start = StartupData::default();

    assert!(start.is_valid());
}

#[test]
#[ignore]
fn test_is_valid_condition_date_all_dates() {
    let mut start_up = StartupData::default();

    start_up.start_date = CtrlTime::from_utc_parts(2022, 1, 1, 0, 0, 0);

    let start = start_up.start_date.0 / GIGA_U;
    let end = CtrlTime::from_utc_parts(2070, 12, 31, 23, 59, 59).0 / GIGA_U;
    for i in start..end {
        start_up.start_date.add_secs(i);
        assert!(start_up.is_valid(), "{}", start_up.start_date.as_rfc3339_str_e());
    }
}

#[test]
fn test_is_valid_condition_date_some_dates() {
    let mut start_up = StartupData::default();

    start_up.start_date = CtrlTime::from_utc_parts(2023, 1, 1, 0, 0, 0);

    let start = start_up.start_date.0 / GIGA_U;
    let end = CtrlTime::from_utc_parts(2024, 12, 31, 23, 59, 59).0 / GIGA_U;
    for i in start..end {
        start_up.start_date.add_secs(i);
        assert!(start_up.is_valid(), "{}", start_up.start_date.as_rfc3339_str_e());
    }
}

#[test]
fn test_is_not_valid_condition_simulation() {
    let mut start = StartupData::default();

    start.start_date = CtrlTime::from_utc_parts(2020, 1, 1, 0, 0, 0);
    assert!(!start.is_valid());
}

#[test]
fn test_is_not_valid_condition_date() {
    let mut start = StartupData::default();

    start.start_date = CtrlTime::from_utc_parts(2020, 1, 1, 0, 0, 0);
    assert!(!start.is_valid());

    start.start_date = CtrlTime::from_utc_parts(2071, 1, 1, 0, 0, 0);
    assert!(!start.is_valid());
}

#[test]
fn test_app_cfg_set_clean_shutdown() {
    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db);

    let res = app_cfg.set_clean_shutdown();

    assert!(res == 1);
    assert!(app_cfg.shutdown == 1);
    assert!(app_cfg.changed);
}

#[test]
fn test_app_cfg_set_shutdown_not_controlled() {
    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db);

    let res = app_cfg.set_shutdown_not_controlled();

    assert!(res == 0);
    assert!(app_cfg.shutdown == 0);
    assert!(app_cfg.changed);
}

#[test]
fn test_app_cfg_validate_last_shutdown_controller() {
    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db);

    let res = app_cfg.set_clean_shutdown();

    assert!(res == 1);
    assert!(app_cfg.was_last_shutdown_controlled());
}

#[test]
fn test_app_cfg_set_live_since() {
    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db);

    let time = CtrlTime::from_utc_parts(2022, 3, 5, 9, 0, 0);
    app_cfg.set_live_since(time);

    assert!(time.as_rfc3339_str_e() == app_cfg.live_since);
}

#[test]
fn test_topic_from_string() {
    assert!(Topic::STC_WEATHER == Topic::from_string(TOPIC[0]));
    assert!(Topic::STC_SYNC_DB == Topic::from_string(TOPIC[1]));
    assert!(Topic::STC_SND_FULLDB == Topic::from_string(TOPIC[2]));
    assert!(Topic::STC_SND_ALERT == Topic::from_string(TOPIC[3]));
    assert!(Topic::STC_SND_ALERT_RESET == Topic::from_string(TOPIC[4]));
    assert!(Topic::STC_SND_LOG_ERROR == Topic::from_string(TOPIC[5]));
    assert!(Topic::STC_WEATHER_HIST == Topic::from_string(TOPIC[6]));
    assert!(Topic::CTS_STOP_CYCLE == Topic::from_string(TOPIC[7]));
    assert!(Topic::CTS_STOP_SECTOR == Topic::from_string(TOPIC[8]));
    assert!(Topic::CTS_STATUS_CHANGE_MODE == Topic::from_string(TOPIC[9]));
    assert!(Topic::CTS_STATUS_SHUTDOWN == Topic::from_string(TOPIC[10]));
    assert!(Topic::CTS_FORCE_CYCLE == Topic::from_string(TOPIC[11]));
    assert!(Topic::CTS_FORCE_SECTOR == Topic::from_string(TOPIC[12]));
    assert!(Topic::CTS_GET_FULLDB == Topic::from_string(TOPIC[13]));
    assert!(Topic::CTS_SYNC_DB == Topic::from_string(TOPIC[14]));
    assert!(Topic::CTS_GET_WEATHER_HIST == Topic::from_string(TOPIC[15]));
    assert!(Topic::CLIENT_CONNECTION == Topic::from_string(TOPIC[16]));
    assert!(Topic::SERVER_CONNECTION == Topic::from_string(TOPIC[17]));
    // assert!(Topic::DEVICE_1_CONNECTION == Topic::from_string(TOPIC[18]));
    // assert!(Topic::NULL == Topic::from_string(TOPIC[19]));
}

#[test]
fn test_topic_from_string_invalid_topics() {
    assert!(Topic::SERVER_CONNECTION == Topic::from_string("a"));
    assert!(Topic::SERVER_CONNECTION == Topic::from_string("iyuiyouoidoasudoudoauids"));
}
