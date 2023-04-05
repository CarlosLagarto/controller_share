//! This service listens for events that are of the clients applications interest (as defined by me :-) )
//! as these are situations that may imply access to every any other service, I am in doubt if this should be an autonomous service
//! or if this should be handled from the application central control point PoV

use arrayvec::ArrayVec;

use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*, tm::*};
use crate::data_structs::client::{db_sync::*, sync_op::*, cycle_cli::*, sector_cli::*};
use crate::data_structs::msgs::{alert::*, ext_message::*, log_error::*, topic::*, weather::*, int_message::*};
use crate::data_structs::rega::{command::*, watering_status::*, mode::*};
use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::client::db_model::*;
use crate::services::irrigation::{cycle_type::*, sector_run::*, wtr_engine::*, wtr_history::*, wtr_svc::*, cycle::*};
use crate::services::weather::{weather_svc::*, weather_history::*};
use crate::services::msg_broker::msg_brkr_svc::*;
use crate::config::{app_config::*, wtr_cfg::*};
use crate::{log_error, log_info, log_warn, logger::*};

use ctrl_prelude::{globals::*, string_resources::*};

/// Dimension 136
pub struct ClntSvc {
    pub schedule: Schedule,
    //if the vector grows, means lots of errors, meaning a bigger problem than used memory...
    errors_to_send: Vec<String>,
    pub last_save: CtrlTime,

    prev_day: Option<CtrlTime>,
    o_week_rain: Option<f32>,
    res_data: Option<Result<DailyWeatherFromDB, DBError>>,
}

impl ClntSvc {
    #[inline]
    pub fn new(app_cfg: &AppCfg, now_time: CtrlTime) -> Self {
        let client_update_interval = app_cfg.check_client_interval as u16;
        let schedule = Schedule::build_run_forever(now_time, client_update_interval, ScheduleRepeatUnit::Seconds);
        Self {
            schedule,
            errors_to_send: ([]).to_vec(),
            last_save: CtrlTime(0),
            o_week_rain: None,
            res_data: None,
            prev_day: None,
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn verify_things_to_do(&mut self, msg_brkr: &MsgBrkr) { self.send_error_if_there_are_any(msg_brkr); }

    #[inline]
    #[rustfmt::skip]
    pub fn add_error(&mut self, err: LogError) { self.errors_to_send.push(err.error); }

    /// Get message, process it, and responde or act on system status - whatever the case
    ///
    /// It is considered:
    /// - full db request
    /// - get db updates after timestamp X
    /// - set partial db updates info from the client regarding (cycles, sectors, configuration)
    /// - and water machine commands/state that are passeed directly to the water machine
    /// - shutdown is sent to the dispatcher to be captured & handled by the main thread.
    ///
    /// In practice the wather machine could listen for this messages and handle them direcly.
    ///
    /// We just thought that in this way we have all client communications in one place, which may ease maintenance...
    /// and add a level of indirection that may allow in the future evolve to some kind of authentication with limited
    /// impact on the code base
    ///
    /// we hope, and also maintain all the things related with clients interface in one place
    /// abstracting / reducing / isolating this info from the other modules
    ///
    /// 26/12/2021  - after a few months of pause, it seems that the "centralization" gets in the way of the mod interdependency.
    /// started moving each message top be handled closer to the subject in order to avoid circular dependencies, reduce the number of
    /// statics, and some API ergonomy review.
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    #[inline]
    /// devolve um bool indicando se tem comandos para a wtr machine
    pub fn process_msg_in(&mut self, ext_msg: &mut ExtMsgIn, msg_brkr: &MsgBrkr, wtr_svc: &mut WtrSvc, wthr_svc: &mut WthrSvc, app_cfg: &mut AppCfg, time: CtrlTime, db: &Persist)->bool {
        let mut have_wtr_machine_cmd = false;
        match ext_msg {
            ExtMsgIn::Cycle(msg)=>{
                match msg.header.as_ref().unwrap().topic{
                    Topic::CTS_STOP_CYCLE => {
                        wtr_svc.snd_command(Command::StopCycle(msg.cycle_id));
                        have_wtr_machine_cmd = true;
                    },
                    Topic::CTS_FORCE_CYCLE => {
                        wtr_svc.snd_command(Command::ForceCycle(msg.cycle_id));
                        have_wtr_machine_cmd = true;
                    }
                    _ =>(),
                };
            },
            ExtMsgIn::Sector(msg)=>{
                match msg.header.as_ref().unwrap().topic{
                    Topic::CTS_STOP_SECTOR => {
                        wtr_svc.snd_command(Command::StopSector(msg.running_ptr.clone()));
                        have_wtr_machine_cmd = true;
                    },
                    Topic::CTS_FORCE_SECTOR => {
                        wtr_svc.snd_command(Command::ForceSector(msg.running_ptr.sec_id.unwrap()));
                        have_wtr_machine_cmd = true;
                    },
                    _ => (),
                }
            }
            ExtMsgIn::ChangeMode(msg)=> {
                wtr_svc.snd_command(Command::ChangeMode(msg.mode));
                have_wtr_machine_cmd = true;
            },
            //this one is indirect - have a listener in the main thread
            ExtMsgIn::ShutDown(_shutdown_msg)=> msg_brkr.snd_shut_down(),
            ExtMsgIn::WeatherHistory(history_req)=> self.get_weather_history(history_req, msg_brkr, db, time),
            ExtMsgIn::DBSync(msg)=> self.sync_client_data(&mut msg.db_sync, msg_brkr, wtr_svc, wthr_svc, app_cfg, time),
            ExtMsgIn::GetFullDB(_msg) => self.get_full_db(msg_brkr, wtr_svc, wthr_svc, app_cfg, time),
            _ => {} //remaining message topic is nop (client connection msg), or is handled directly by mqtt service, or is water db sync in water service
        }
        have_wtr_machine_cmd
    }

    #[inline]
    fn send_error_if_there_are_any(&mut self, msg_brkr: &MsgBrkr) {
        let mut error_msg: String;
        while !self.errors_to_send.is_empty() {
            error_msg = self.errors_to_send.pop().unwrap();
            msg_brkr.snd_ext_msg(LogError::new_out(error_msg, CtrlTime::sys_time()));
        }
    }

    #[inline]
    pub fn process_water_history(&mut self, msg_brkr: &MsgBrkr, time: CtrlTime, db: &Persist) {
        let wtr_history_to_return: WaterHstry;
        if let Some(data) = WaterHstry::build(time.ux_ts(), db) {
            wtr_history_to_return = data;
        } else {
            let msg = "Histórico da rega sem informação nos ultimos 15 dias".to_owned();
            log_warn!(&msg);
            wtr_history_to_return = WaterHstry::new();
        }
        msg_brkr.reg_int_msg(MsgData::RspWateringHistory(wtr_history_to_return), time);
    }

    #[inline]
    #[rustfmt::skip]
    fn get_weather_history(&mut self, ext_msg: &WeatherHstryMsg, msg_broker: &MsgBrkr, db: &Persist, time: CtrlTime) {
        if let Some(data) = WeatherHstry::build(ext_msg.header.as_ref().unwrap().time, db, ext_msg.uuid()) {
            let history_msg = WeatherHstry::new_out(ExtMsgOut::WeatherHistory(Box::new(data)), time);
            msg_broker.snd_ext_msg(history_msg);
        } else {
            let msg = ERR_GETTING_METEO_HISTORY.to_owned();
            log_error!(&msg);
            self.add_error(LogError { header: None, error: msg, });
        }
    }
    
    #[inline]
    #[rustfmt::skip]
    pub fn sync_client_data( &mut self, data: &mut DBSync, msg_brkr: &MsgBrkr, wtr_svc: &mut WtrSvc, whtr_svc: &mut WthrSvc, app_cfg: &mut AppCfg, time: CtrlTime) {
        // GUARD - // If not in manual mode = do nothing
        // The GUARD simplifies the logic bellow. (one less level of identation at least)
        {
            if !wtr_svc.get_mode().is_manual() { return; }
        }
        // Every client should know what status the machine is.
        // If someone wants to change the machine configuration, need to put the machine in manual mode
        // This message only handles data changes.  Machine status is directly handled by other messages.
        let mut changed = false;

        app_cfg.last_client_update = CtrlTime::from_ux_ts(data.last_sync);
        let mut cycle_cli: CycleCli;
        let mut cycle: Cycle;
        let mut result : Result<Option<CtrlTime>, ScheduleError>;
        let mut msg: String;
        while !data.cycles.is_empty() {
            // client only changes, at the present (I may change my mind later on), cycle schedule related info
            cycle_cli = data.cycles.pop().unwrap();
            cycle = cycle_cli.from_client();
            match cycle_cli.op {
                SyncOp::I => {
                    if wtr_svc.engine.is_new_cycle(cycle_cli.run.cycle_id) {
                        
                        result = find_next_event(time, &cycle.schedule);
                        match result {
                            Ok(Some(next_event_ts)) if next_event_ts > time => {
                                log_info!(INFO_CYCLE_ADD);
                                wtr_svc.add_cycle_from_client(cycle, next_event_ts);
                                changed = true;
                            }
                            Err(err) => {
                                let msg = &err.to_string();
                                log_error!(&msg);
                                msg_brkr.snd_error_to_client(msg);
                            }
                            _ => {} //nop
                        }
                    } else {
                        msg = WARN_CLI_DUPLICATED_CYCLE.to_owned();
                        log_error!(&msg);
                        msg_brkr.snd_error_to_client(&msg);
                    }
                }
                SyncOp::D => match wtr_svc.delete_cycle_from_client(cycle_cli.run.cycle_id) {
                    Ok(_) => {
                        log_info!(INFO_CYCLE_DEL);
                        changed = true;
                    }
                    Err(err) => {
                        log_error!(err.to_string());
                        msg_brkr.snd_error_to_client(WARN_CLI_DEL_NOT_EXISTING_CYCLE);
                    }
                },
                SyncOp::U => match wtr_svc.update_cycle_from_client(&cycle, time) {
                    Ok(_) => {
                        log_info!(INFO_CYCLE_UPD);
                        changed = true;
                    }
                    Err(err) => {
                        msg = err.to_string();
                        log_error!(msg);
                        msg_brkr.snd_error_to_client(&msg);
                    }
                },
            }
        }//end while
        let sector_change = !data.sectors.is_empty();

        for sector_from_cli in &data.sectors {
            wtr_svc.update_physical_sector_from_client(sector_from_cli)
        }
        //this is outside the if bellow so it is in the lock block to avoid deadlock
        //probability of the cycles and/or cyclesectors access sectors info is greater than zero...i think :-)
        changed |= sector_change;

        if sector_change {
            wtr_svc.save_physical_sector_list();
            log_info!(INFO_SECTORS_UPD);
        }
        // process config data from the client, if changes requested
        if let Some(cfg) = &data.config {
            log_info!(INFO_CONFIG_UPD);
            let config = &mut wtr_svc.engine.wtr_cfg;
            let mut wthr_cfg = whtr_svc.get_context();
            app_cfg.from_client(cfg, config, &mut wthr_cfg);
            changed = true;
        }
        if changed {
            wtr_svc.snd_command(Command::ChangeState);
        }
        self.get_full_db(msg_brkr, wtr_svc, whtr_svc, app_cfg, time);
    }

    #[inline]
    pub fn process_alert(&self, alert: Alert, msg_broker: &MsgBrkr, time: CtrlTime) {
        let alert_msg = Alert::new_out(ExtMsgOut::Alert(alert), time);
        msg_broker.snd_ext_msg(alert_msg);
    }

    #[inline]
    pub fn process_weather(&mut self, mut weather: Box<Weather>, msg_broker: &MsgBrkr, time: CtrlTime, db: &Persist) {

        let o_daily_rain = db.get_daily_rain(time);

        if let Some(prev_day) = self.prev_day{
            if prev_day != time.sod_ux_e(){
                self.o_week_rain = db.get_week_acc_rain(time);
                self.res_data = Some(db.get_daily_data(time));
            }
        }else{
            // running for the first time.  Get data
            self.o_week_rain = db.get_week_acc_rain(time);
            self.res_data = Some(db.get_daily_data(time));
        }
        if let Some(daily_rain) = o_daily_rain{
            weather.rain_today = daily_rain;
        }
        if let Some(week_rain) = self.o_week_rain{
            weather.rain_week_acc = week_rain;
        }
        if let Some(Ok(data)) = &self.res_data{
            weather.rain_class_forecast = data.rain_class_forecast.round() as u8;
            weather.rain_probability = data.rain_probability;
            weather.et = data.et;
        }
        
        let weather_msg = ExtMsgOut::new(Topic::STC_WEATHER, ExtMsgOut::Weather(weather), time);
        msg_broker.snd_ext_msg(weather_msg);
    }

    // always send full db.  few kb and simplifies the logic and the multiple exceptions
    #[inline]
    pub fn send_changes_to_client(&mut self, time: CtrlTime, msg_brkr: &MsgBrkr, wtr_svc: &WtrSvc, wthr_svc: &WthrSvc, app_cfg: &mut AppCfg) {
        self.get_full_db(msg_brkr, wtr_svc, wthr_svc, app_cfg, time);
    }

    #[inline]
    fn get_full_db(&mut self, msg_brkr: &MsgBrkr, wtr_svc: &WtrSvc, wthr_svc: &WthrSvc, app_cfg: &mut AppCfg, time: CtrlTime) {
        let db_sync: DBSync;
        {
            let cycle_list = &wtr_svc.cycles_clone();
            let sec_list = &wtr_svc.engine.sectors;
            let wtr_cfg = &wtr_svc.engine.wtr_cfg;
            let mut cycle_cli_list: CycleCliList = cycle_list_to_cli(cycle_list);
            let (first_cycle_time, first_cycle_pos) = get_first_cycle_time(&cycle_cli_list, wtr_svc.get_mode());
            let mut last_sec_end: CtrlTime = CtrlTime(0);
            let sec_cli_list: SecCliList = sec_list_to_cli(sec_list, &wtr_svc.engine.run_secs, first_cycle_time, &wtr_svc.engine.db, wtr_cfg, &mut last_sec_end);
            if (first_cycle_pos as usize) < cycle_cli_list.len(){
                cycle_cli_list[first_cycle_pos as usize].run.end = last_sec_end.ux_ts();
            } 
            // else{ // in manual mode there is no cycles to execute
            db_sync = DBSync::build_full(time, &app_cfg.to_client(&wtr_svc.engine.wtr_cfg, wthr_svc.get_context()),cycle_cli_list, sec_cli_list);
        }
        // previous code block is to assure that any locks to the msgbroker are cleaned/dropped before send_sync_msg
        send_sync_msg(app_cfg, time, db_sync, msg_brkr);
    }
}

/// Returns (time of the first cycle to water, list index) <br>
/// Ignores direct cycles, except if we have one active . <br>
/// Ignores manual mode, except if we have one active <br>
/// Meaning, in manual mode always returns CtrlTime(u64::MAX), meaning no cycle to execute unless we have an active cycle <br>
///  <br>
/// Resume,  <br>
///     - when in standard mode returns next cycle time, if there is one defined <br>
///     - when in wizard mode return next cycle time (in wizard mode we always have a defined cycle) <br>
#[inline]
fn get_first_cycle_time(cycle_cli_list: &CycleCliList, mode: Mode) -> (CtrlTime, i8) {
    let mut first_cycle_time = u64::MAX;
    let mut first_cycle_id = -1;
    for (pos, cycle) in cycle_cli_list.iter().enumerate() {
        // 99.9% enters here 
        if cycle.run.status != WateringStatus::Running{
            // only care for not direct and not compensation cycles
            if cycle.cycle_type == CycleType::Standard as u8 && mode== Mode::Standard && cycle.schedule.start < first_cycle_time{
                first_cycle_time = cycle.schedule.start;
                first_cycle_id = pos as i8;    
            }
            if cycle.cycle_type == CycleType::Wizard as u8 && mode== Mode::Wizard && cycle.schedule.start < first_cycle_time{
                first_cycle_time = cycle.schedule.start;
                first_cycle_id = pos as i8;    
            }
        }else{
            // is a cycle is running, only that cycle info is returned
            first_cycle_time = cycle.schedule.start;
            first_cycle_id = pos as i8;
            break; // and as by design we only one cycle running at each moment, we leave the cycle
        }
    }
    if first_cycle_time == u64::MAX{
        first_cycle_time = 0;
    }
    (CtrlTime::from_ux_ts(first_cycle_time), first_cycle_id)
}

#[inline]
#[rustfmt::skip]
fn sec_list_to_cli(sec_list: &SecList, run_secs: &SecRunList, cycle_start: CtrlTime, db: &Persist, wtr_cfg: &WtrCfg, last_sec_end: &mut CtrlTime) -> SecCliList {
    let mut sec_cli_list: SecCliList = ArrayVec::<SectorCli, MAX_SECTORS>::new();
    let mut sec_start = cycle_start;
    let mut run_sec: SectorRun;
    let mut run_sec_ref: &SectorRun;
    let mut is_first_sec = true;
    let mut pump_recycle_time: f32 = 0.;
    let def_pump_recycle_time = wtr_cfg.pump_recycle_time as f32;

    last_sec_end.0 = 0;

    for sec in sec_list.iter() {
        if sec.enabled {
            // look in the run_secs
            let res_pos = run_secs.binary_search_by(|run_sec| run_sec.sec_id.cmp(&sec.id));
            if let Ok(pos) = res_pos {
                //  if ok, means that is watering and in that case data structs are updated
                run_sec_ref = &run_secs[pos];
            } else {
                // if nok, means no cycles are running, so we create a dummy to provide for client info
                run_sec = SectorRun::new(0, 0, sec.id, WateringStatus::Waiting);
                let (duration_minutes, _) = calc_dur_and_deficit(wtr_cfg, &run_sec, db, sec, sec_start);
                if !is_first_sec {
                    pump_recycle_time = def_pump_recycle_time;
                }
                upd_run_sec_data(&mut run_sec, sec_start, 0, 0, duration_minutes, pump_recycle_time);
                run_sec_ref = &run_sec;
                sec_start = sec_start.add_secs_f32(min_to_sec_f32(duration_minutes));
            }
            is_first_sec = false;
        } else {
            // when sector disabled, sector will not get water so we can send default info
            run_sec = SectorRun::default();
            run_sec_ref = &run_sec;
        }
        // if no active watering  last_sec_end = 0
        last_sec_end.0 = last_sec_end.0.max(run_sec_ref.end.0);
        sec_cli_list.push(SectorCli::to_client(sec, run_sec_ref));
    }
    sec_cli_list
}

#[inline]
fn cycle_list_to_cli(cycle_list: &CycleList) -> CycleCliList {
    let mut cycle_cli_list: CycleCliList = ArrayVec::<CycleCli, MAX_CYCLES>::new();

    for cycle in cycle_list.iter() {
        cycle_cli_list.push(CycleCli::to_client(cycle))
    }
    cycle_cli_list
}

#[inline]
fn send_sync_msg(app_cfg: &mut AppCfg, time: CtrlTime, data: DBSync, msg_brkr: &MsgBrkr) {
    app_cfg.last_client_update = time;
    let full_db_msg = DBSync::new_out(data, time);
    msg_brkr.snd_ext_msg(full_db_msg);
}
