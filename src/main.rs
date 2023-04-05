use std::net::TcpListener;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Instant;
/// ----------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// INTRODUCTION
/// 
/// controller is a program to control stuff
///
/// The stuff to control are:
/// - lawn watering
/// - weather
/// - sensors
///
/// In order to do this several modules are developed that provide information and support to the intended functionality.
///
/// The support modules are:
/// - weather - that have to flavours.  
///     - The first one gathers information from a web provider to feed the system.
///     - The second one is the weather station to autonomously gather weather information onsite
///   as of 2022/mar/15  only the first flavour is implemented
///   as of 2022/Oct/07  the second flavour is implemented - station arrived during last summer
/// - web - that provides a http channel to remotely access/control the program with a few selected basic commands
/// - mqtt - that provides a mqtt channel that on top of beeing a channel to remotely control the controller,
///          also its the only channel to feed any client that follows the defined protocol, and to subscribe other remote brokers for sensors
/// - irrigation - that implements the state machine programmed to water the lawn
/// - msg broker - that is a internal event manager to exchange information between the internal threads
/// - client service that control the state of the client.  
///     - It is assumed that there is only one client active in each moment.  
///     - This is intended to be used only by one client, me, so although it can be used from several terminals, it is assumed that only one will be active at each moment
/// - electronics - that implements the base functionality to interact with the physical parts of the system (watering relays and physical sensors)
///
/// A few things had pop up while developing the program.
///
/// 1. although best practices recommend to not having static stuff, web and mqtt by their very nature, that is, control an I/O port that is unique
///     within the computer, cannot have several instances accessing to the same port.  So, having several instances makes no sense in this use case.
///     We could change the port for each instance, implying to have a different configuration for each instance, or make a static shared object
///     I did implement the shared static approach and special care (single thread) is taken in tests
///
/// 
/// The controler only runs in single instance in each defined environment (DEV and PRD).
/// MQTT test topics have "/TEST" suffix, with same address for broker
/// Web server service for the backend command processor is 5003 and 5004, at [www]controller_direct/<comando> and [www]/controller_direct_test/<comando>
/// The Apache server proxy to the right port depending on the url
/// 
/// ----------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// 
/// Things to keep in mind for the configuration
/// 
/// db_config.toml - database configuration
/// log_config.toml - log configuration, and also if we are on TEST or PRD environment
/// mqtt_config.toml - mqtt configuration
/// web_rest_config.toml - web service configuration
/// 
/// Frontend apache proxy to the backend all calls to controller_direct and controller_direct_test to ports 5003 e 5004 
/// Frontend apache is cinfigured with two way SSL authentication at port 443, so all browsers and server need certificates (issued by me)
/// 
/// Internet router have to allow access (port forward) to 443 and to the websocket configured in mqtt
/// 
/// dw.db is hard coded in the scripts REVIEW.  Operational db can have whatever name (with the defined schema), but should be in the same place...
/// 
/// ----------------------------------------------------------------------------------------------------------------------------------------------------------------------

use controller::{run::*, signal_handler::*};
use ctrl_lib::config::log_config::*;
use ctrl_lib::{log_info, logger::*, utils::*};
use ctrl_prelude::string_resources::*;

const EXIT_CODE_DUPLICATED_INSTANCE: i32 = 1;

const SINGLE_INSTANCE_ADDRESS: &str = "0.0.0.0";
const SINGLE_INSTANCE_PORT: u16 = 12345;
const SINGLE_INSTANCE_TEST_PORT: u16 = 12346;

fn main() {
    let t0 = Instant::now();
    better_panic::install();

    let app_log = AppLog::new();
    let logger_handle = initialize_logger(&app_log);

    if app_log.test_in_progress == 1 {
        unsafe { TESTING = true };
    }
    // control a single instance of the application
    let lock_port: u16 = if unsafe { TESTING } { SINGLE_INSTANCE_PORT } else { SINGLE_INSTANCE_TEST_PORT };

    let lock_socket = TcpListener::bind((SINGLE_INSTANCE_ADDRESS, lock_port)).unwrap_or_else(|_| {
        eprintln!("Another controller instance is already running");
        std::process::exit(EXIT_CODE_DUPLICATED_INSTANCE);
    });

    let exit_code = Arc::new(AtomicI32::new(0));
    let inner_code = exit_code.clone();
    std::thread::spawn(move || listen_interrupt(&inner_code));
    if exit_code.load(Ordering::Relaxed) != EXIT_CODE_CANT_INSTALL_HANDLER {
        exit_code.store(run(), Ordering::Relaxed);
    } else {
        eprintln!("Could not install the signal handler.");
    }
    // remove application instance lock
    drop(lock_socket);
    log_info!(INFO_LAST_WORDS);
    // exit controller program
    info!("Executed. Ran for {}", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    logger_handle.shutdown();
    LoggerHandle::flush(&logger_handle);
    let code = exit_code.load(Ordering::Relaxed);
    std::process::exit(code)
}
