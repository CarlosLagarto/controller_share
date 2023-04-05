use std::{any::Any, sync::Arc};

use ctrl_lib::app_context::start_up::*;
use ctrl_lib::data_structs::concurrent_queue::*;
use ctrl_lib::services::electronics::devices_svc::DevicesSvc;
use ctrl_lib::services::msg_broker::{msg_brkr_svc::*, subscriber::*};
use ctrl_lib::services::{mqtt::mqtt_svc::*, weather::weather_svc::*, web_svc::*};
use ctrl_lib::{app_time::ctrl_time::*, config::app_config::*, db::db_sql_lite::*};
use ctrl_lib::{controller_sync::*, log_debug, log_info, log_warn, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};
use crate::control::process_control;

const EXIT_CODE_OK: i32 = 0;
// plus other codes from "install_signal_handler"

#[rustfmt::skip]
#[inline]
pub fn run() -> i32 {
    log_info!(INFO_PROGRAM_STARTED);
    let db = Persist::new();
    let mut app_cfg = AppCfg::new(db.clone());

    app_cfg.start_up = StartupData::default();
    let start_time = app_cfg.start_up.start_date;
    setup_start_time(start_time);
    app_cfg.set_live_since(start_time);

    // initialize thread coordination conditions signals
    DB_MAINT_SIG.read().set();
    NEW_DAY_SIG.read().set();

    verify_last_shutdown(&mut app_cfg);

    // services creation and start order is very important due to the necessary thread coordination and wait events/conditions
    // REVIEW:, o que fazer se as threads não arrancarem - rever o error handling
    // We can live only with msg_brkr e web_service threads (meaning, live without mqtt, client web and weather)
    log_info!(INFO_STARTING_MAIN_CHECK);
    // initialize service handlers
    let main_subs_queue = Arc::new(MtDeque::new());
    let brkr_svc = Arc::new(MsgBrkr::new());
    brkr_svc.register_in_broker(Subscriber::Main, main_subs_queue.clone());
    let handle_msg_broker = brkr_svc.start();

    let dev_svc = Arc::new(DevicesSvc::new(&db.clone(), brkr_svc.clone()));
    let handler_dev = dev_svc.start();

    let web_svc = WebService::default();
    let handler_web = web_svc.start(brkr_svc.clone());

    let mut mqtt_svc = MQTTService::new(brkr_svc.clone());
    let handler_mqtt = mqtt_svc.start();

    // have to be called before the water service, dependency of meteo alerts in wizard mode)
    let mut wthr_svc = WthrSvc::new( app_cfg.start_up.start_date, db.clone(), );
    let handler_wthr = wthr_svc.start(app_cfg.time.time_control_interval as u64, db.clone(), brkr_svc.clone());

    // SPRINT SENSORES - aqui entrarão os sensores

    // inside there is a loop that exits when shutdown event is received.
    let exit_time = process_control(&mut app_cfg, brkr_svc.clone(), &mut wthr_svc,  &db, start_time, main_subs_queue, dev_svc.clone());

    // shutdown all the services in a controlled way
    dev_svc.terminate();
    handle_join_result(handler_dev.join(), DEV_SERVICE_THREAD);
    wthr_svc.terminate();
    handle_join_result(handler_wthr.join(), WTHR_SERVICE_THREAD);
    web_svc.terminate();
    handle_join_result(handler_web.join(), WBSR_SERVICE_THREAD);
    mqtt_svc.terminate();
    handle_join_result(handler_mqtt.join(), MQTT_SERVICE_THREAD);

    brkr_svc.terminate();
    handle_join_result(handle_msg_broker.join(), EVBR_SERVICE_THREAD);

    // BD closes it self on objects/variables drop
    // housekeeping
    app_cfg.set_clean_shutdown();

    log_info!(INFO_SHUTDOWN_COMPLETED);

    app_cfg.save_if_updated(exit_time);
   
    EXIT_CODE_OK
}

#[inline]
#[rustfmt::skip]
fn handle_join_result(res: Result<(), Box<dyn Any + Send>>, thread_name: &str) {
    log_debug!(dbg_join_end(thread_name));
    if let Err(err) = res { 
        error!("{:?}", err); 
    };
}

#[inline]
fn verify_last_shutdown(app_cfg: &mut AppCfg) {
    if app_cfg.was_last_shutdown_controlled() {
        log_warn!(WARN_LAST_SHUTDOWN_UNCONTROLLED);
        log_warn!(WARN_TRYING_TO_RECOVER);
    } else {
        app_cfg.set_shutdown_not_controlled();
    }
}
