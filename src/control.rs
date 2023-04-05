use ctrl_lib::data_structs::{concurrent_queue::*, msgs::int_message::*, rega::state::*};
#[cfg(debug_assertions)]
use ctrl_lib::log_debug;
use ctrl_lib::services::electronics::devices_svc::*;
use ctrl_lib::services::{client::clients_svc::*, msg_broker::msg_brkr_svc::*, msg_broker::subscriber::*, weather::weather_svc::*};
use ctrl_lib::services::{db_maint::db_mnt_svc::*, irrigation::wtr_svc::*};
use ctrl_lib::utils::get_deadline_instant;
#[cfg(debug_assertions)]
use ctrl_lib::utils::THREAD_COUNT;
use ctrl_lib::{app_time::ctrl_time::*, config::app_config::*, controller_sync::*, db::db_sql_lite::*};
use ctrl_lib::{log_info, log_warn, logger::*, string_concat::*};
use ctrl_prelude::{globals::*, string_resources::*};

#[inline]
#[rustfmt::skip]
pub fn process_control(app_cfg: &mut AppCfg, msg_brkr: SMsgBrkr, wthr_svc: &mut WthrSvc, db: &Persist, start_time: CtrlTime, main_subs_queue: SMtDeque, dev_svc: SDevicesSvc) -> CtrlTime {
    //subscribe broker events
    msg_brkr.subscribe(MsgType::ShutDown, Subscriber::Main); // end loop
    msg_brkr.subscribe(MsgType::Alert, Subscriber::Main); // inform client
    msg_brkr.subscribe(MsgType::MessageIn, Subscriber::Main); // inform client
    msg_brkr.subscribe(MsgType::ClientError, Subscriber::Main); // inform client
    msg_brkr.subscribe(MsgType::Command, Subscriber::Main); // inform water machine
    msg_brkr.subscribe(MsgType::CycleAdded, Subscriber::Main); // inform water machine 
    msg_brkr.subscribe(MsgType::Weather, Subscriber::Main); // inform water machine and clients
    msg_brkr.subscribe(MsgType::StateChanged, Subscriber::Main); // inform client   
    msg_brkr.subscribe(MsgType::GetWateringHistory, Subscriber::Main); // inform client   

    let mut ctrl_time: CtrlTime = start_time;

    let mut db_mnt_svc = DBMntSvc::new(ctrl_time,  app_cfg);   
    let mut clnt_svc = ClntSvc::new(app_cfg, ctrl_time,);

    let alert_thresholds = wthr_svc.get_context().read().alrt_thresholds.clone();
    let mut wtr_svc = WtrSvc::new(alert_thresholds, msg_brkr.clone(), db.clone(), ctrl_time, dev_svc);

    let interval = app_cfg.time.time_control_interval as u64 * GIGA_U;
    let mut last_state: State = wtr_svc.state();
    let mut have_wtr_machine_cmd: bool;

    loop {    
        have_wtr_machine_cmd = false;
        // time is adjusted to the next second, following the configured interval
        let result_tuple  = main_subs_queue.recv_timeout(get_deadline_instant(interval));
        match result_tuple{
            (Some(i_msg), _, _) =>{
                // we have a msg to process
                // wait if processing new day or db maint.
                new_day_and_db_mnt_sync();
                ctrl_time = CtrlTime::sys_time();
                match i_msg.data {
                    MsgData::Alert(alert) => {
                        wtr_svc.process_alert(alert.clone()); // priority to water machine
                        have_wtr_machine_cmd = true;
                        clnt_svc.process_alert(alert, &msg_brkr, ctrl_time); //call client info
                    }                    
                    MsgData::MessageIn(mut ext_i_msg) =>  {
                        have_wtr_machine_cmd = clnt_svc.process_msg_in(&mut ext_i_msg, &msg_brkr, &mut wtr_svc, wthr_svc,  app_cfg, ctrl_time, db);
                    },
                    MsgData::ClientError(log_error) => clnt_svc.add_error(log_error.clone()), //call client info
                    // the only way to shutdown is via web api rest - who in turn sends msg to the broker, and the broker notifies all subscribers
                    MsgData::ShutDown(_) =>  break,  // loop exit - exits main listener.
                    MsgData::Command(cmd) => {
                        wtr_svc.process_command(cmd, ctrl_time); // call water machine
                        have_wtr_machine_cmd = true;
                    },
                    MsgData::CycleAdded => {
                        wtr_svc.process_cycle_added();
                        have_wtr_machine_cmd = true;
                    },
                    MsgData::Weather(weather) => {
                        wtr_svc.process_weather(&weather);
                        have_wtr_machine_cmd = true;
                        clnt_svc.process_weather(Box::new(weather), &msg_brkr, ctrl_time, db)
                    }
                    // outside notification are decided by the water machine
                    // internal changes, are all managed byt the external or internal commands applied
                    // so, if state changed, we should notify the clients
                    MsgData::StateChanged => {
                        clnt_svc.send_changes_to_client(ctrl_time, &msg_brkr, &wtr_svc, wthr_svc, app_cfg);
                        if wtr_svc.state() != last_state{
                            log_info!(string_concat!(INFO_WTR_STATE_INFO_1P, &wtr_svc.state().to_string()));
                            last_state = wtr_svc.state();
                        }
                    }
                    MsgData::GetWateringHistory => {
                        clnt_svc.process_water_history(&msg_brkr, ctrl_time, db);
                    }
                    _ => log_warn!(warn_unsubs_msg_type(&i_msg.data.to_string())),
                }
            },
            (None, is_time_out, _) => {
                if is_time_out{
                    // log_debug!(dbg_time_tick(&ctrl_time.as_rfc3339_str_e()));
                    verify_things_to_do(&mut wtr_svc, &mut db_mnt_svc, &mut clnt_svc,  &msg_brkr, app_cfg, ctrl_time);
                }else{
                    break; // if not timeout, then is shutdown
                }
            },
        }
        if have_wtr_machine_cmd{
             wtr_svc.verify_things_to_do(CtrlTime::sys_time());
        }
    } //loop 
    // we may exit from msg_broker msg or from ctrl c handler
    log_info!(info_prgrm_starting_shutdown(&wtr_svc.state().to_string()));
    unsafe{SHUTTING_DOWN = true};  // help context to know that we are terminating

    #[cfg(debug_assertions)]
    { unsafe { log_debug!(dbf_nr_of_active_threads(THREAD_COUNT)); } }
    ctrl_time
}

#[inline]
pub fn verify_things_to_do(wtr_svc: &mut WtrSvc, db_mnt_svc: &mut DBMntSvc, clnt_svc: &mut ClntSvc, msg_brkr: &MsgBrkr, app_cfg: &mut AppCfg, time: CtrlTime) {
    // give priority to the water machine
    wtr_svc.verify_things_to_do(time);
    // and then handle the other services
    db_mnt_svc.verify_things_to_do(time, msg_brkr, app_cfg);
    clnt_svc.verify_things_to_do(msg_brkr);
    app_cfg.save_if_updated(time);
}
