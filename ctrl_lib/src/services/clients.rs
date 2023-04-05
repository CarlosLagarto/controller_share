//! This service listens for events that are of the clients applications interest (as defined by me :-) )
//! as these are situations that may imply access to every any other service, I am in doubt if this should be an autonomous service
//! or if this should be handled from the application central control point PoV

use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::data_structs::client::{client_context::*, db_sync::*, sync_op::*};
use crate::data_structs::msgs::{alert::*, ext_message::*, int_message::*, log_error::*, topic::*, weather::*};
use crate::data_structs::rega::command::*;
use crate::services::weather::{weather_history::*, weather_service::*};
use crate::services::{irrigation::wtr_svc::*, msg_broker::svc::*};
use crate::{config::app_config::*, db::db_sql_lite::*};
use crate::{log_error, log_info, log_warn, logger::*};

use ctrl_prelude::string_resources::*;

/// Dimensão 168
pub struct ClntSvc {
    pub schedule: Schedule,
    //if the vector grows, means lots of errors, meaning a bigger problem than used memory...
    errors_to_send: Vec<String>,
    prev_cfg: ClientContext,
    pub last_save: CtrlTime,
}

impl ClntSvc {
    #[inline]
    pub fn new(app_cfg: &AppCfg, now_time: CtrlTime) -> Self {
        let client_update_interval = app_cfg.check_client_interval as u16;
        let schedule = Schedule::build_run_forever(now_time, client_update_interval, ScheduleRepeatUnit::Seconds);
        Self {
            schedule,
            errors_to_send: ([]).to_vec(),
            prev_cfg: ClientContext::default(),
            last_save: CtrlTime(0),
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn process_time_tick(&mut self, msg_brkr: &MsgBrkr) { self.send_error_if_there_are_any(msg_brkr); }

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
    /// - shutdown and restart are sent to the dispatcher to be captured & handled by the main thread.
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
    pub fn process_msg_in(&mut self, ext_msg: &mut ExtMsg, msg_brkr: &MsgBrkr, wtr_svc: &mut WtrSvc, wthr_svc: &mut WthrSvc, db: &Persist, app_cfg: &mut AppCfg, time: CtrlTime) {
        match ext_msg.topic {
            Topic::CTS_STOP_CYCLE => {
                if let ExtMsgData::Cycle(cycle_data) = ext_msg.data {
                    wtr_svc.snd_command(Command::StopCycle(cycle_data));
                }
            }
            Topic::CTS_STOP_SECTOR => {
                if let ExtMsgData::Sector(running_ptr) = &ext_msg.data {
                    wtr_svc.snd_command(Command::StopSector(running_ptr.clone()));
                }
            }
            Topic::CTS_STATUS_CHANGE_MODE => {
                if let ExtMsgData::ChangeMode(mode) = ext_msg.data {
                    wtr_svc.snd_command(Command::ChangeMode(mode));
                }
            }
            Topic::CTS_FORCE_CYCLE => {
                if let ExtMsgData::Cycle(cycle_data) = ext_msg.data {
                    wtr_svc.snd_command(Command::ForceCycle(cycle_data));
                }
            }
            Topic::CTS_FORCE_SECTOR => {
                if let ExtMsgData::Sector(running_ptr) = &ext_msg.data {
                    wtr_svc.snd_command(Command::ForceSector(running_ptr.sec_id.unwrap()));
                }
            }
            Topic::CTS_STATUS_SHUTDOWN => {
                //this one is indirect - have a listener in the main thread
                msg_brkr.snd_shut_down(DESC_SHUTDOWN_CMD_FRM_CLIENT);
            }
            Topic::CTS_STATUS_RESTART => {
                msg_brkr.reg_int_msg(MsgData::Restart, time, DESC_RESTART_CMD_FROM_CLIENT);
            }
            Topic::CTS_GET_WEATHER_HIST => self.process_weather_history_request(&ext_msg.data, msg_brkr, db, time),

            //Get message, process it, and responde or act on system status - whatever the case
            //We are considering:
            //- full db request
            //- get db updates after timestamp X
            //- set partial db updates info from the client regarding (cycles, sectors, configuration)
            //- and water machine commands/state that are passeed directly to the water machine
            //- shutdown and restart are sent to the dispatcher to be captured & handled by the main thread.
            //In practice the wather machine could listen for this messages and handle them direcly.
            //We just thought that in this way we have all client communications in one place, which may ease maintenance...
            //and add a level of indirection that may allow in the future evolve to some kind of authentication with limited
            //impact on the code base - we hope, and also maintain all the things related with clients interface in one place
            //abstracting / reducing / isolating this info from the other modules
            Topic::CTS_GET_FULLDB => self.send_full_db_to_client(msg_brkr, wtr_svc, wthr_svc, app_cfg, time),
            Topic::CTS_SYNC_DB => self.process_incoming_data(ext_msg, msg_brkr, wtr_svc, wthr_svc, app_cfg, time),

            _ => {} //remaining message topic is nop (client connection msg), or is handled directly by mqtt service, or is water db sync in water service
        }
    }

    #[inline]
    fn send_error_if_there_are_any(&mut self, msg_brkr: &MsgBrkr) {
        let mut error_msg: String;
        while !self.errors_to_send.is_empty() {
            error_msg = self.errors_to_send.pop().unwrap();
            msg_brkr.snd_ext_msg(LogError::new_out(error_msg, CtrlTime::sim_time()));
        }
        // a linha abaixo faz o mesmo que o loop acima, em tese com um pouco mais de performance, mas o loop dá mais jeito para debug
        // self.errors_to_send.drain(..).for_each(|error_msg| msg_brkr.register_message(LogError::new_out(error_msg, CtrlTime::sim_time())));
    }

    #[inline]
    #[rustfmt::skip]
    fn process_weather_history_request(&mut self, ext_msg: &ExtMsgData, msg_broker: &MsgBrkr, db: &Persist, time: CtrlTime) {
        if let ExtMsgData::UxTimestamp(time_stamp) = ext_msg {
            if let Some(data) = WeatherHstry::build(*time_stamp, db) {
                let history_msg = WeatherHstry::new_out(ExtMsgData::WeatherHistory(Box::new(data)), time);
                msg_broker.snd_ext_msg(history_msg);
            } else {
                let msg = ERR_GETTING_METEO_HISTORY.to_owned();
                log_error!(&msg);
                self.add_error(LogError { error: msg, });
            }
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn process_incoming_data( &mut self, ext_msg: &mut ExtMsg, msg_brkr: &MsgBrkr, wtr_svc: &mut WtrSvc, whtr_svc: &mut WthrSvc, app_cfg: &mut AppCfg, time: CtrlTime, ) {
        // GUARD - // If not in manual mode = do nothing
        // The GUARD simplifies the logic bellow. (one less level of identation at least)
        {
            if wtr_svc.get_mode().is_manual() { return; }
        }
        // Every client should know what status the machine is.
        // If someone wants to change the machine configuration, need to put the machine in manual mode
        // This message only handles data changes .  Machine status is directly handled by other messages.
        let mut changed = false;

        if let ExtMsgData::DBSync(data) = &mut ext_msg.data {
            app_cfg.last_client_update = data.last_sync;
            while !data.cycles.is_empty() {
                // client only changes, at the present (I may change my mind later on), cycle schedule related info
                let cycle = data.cycles.pop().unwrap();
                match cycle.op {
                    SyncOp::I => {
                        if wtr_svc.engine.is_new_cycle(cycle.run.cycle_id) {
                            let result = find_next_event(time, &cycle.schedule);
                            match result {
                                Ok(Some(next_event_ts)) if next_event_ts > time => {
                                    log_info!(INFO_CYCLE_ADD);
                                    wtr_svc.add_cycle_from_client(cycle, next_event_ts);
                                    changed = true;
                                }
                                Err(err) => {
                                    log_error!(err.to_string());
                                    msg_brkr.snd_error_to_clients(&err.to_string(), ERR_CLI_CYCLE_INSERT);
                                }
                                _ => {} //nop
                            }
                        } else {
                            let msg = WARN_CLI_DUPLICATED_CYCLE;
                            log_warn!(msg);
                            msg_brkr.snd_error_to_clients(msg, msg);
                        }
                    }
                    SyncOp::D => match wtr_svc.delete_cycle_from_client(cycle.run.cycle_id) {
                        Ok(_) => {
                            log_info!(INFO_CYCLE_DEL);
                            changed = true;
                        }
                        Err(err) => {
                            log_warn!(err.to_string());
                            msg_brkr.snd_error_to_clients(WARN_CLI_DEL_NOT_EXISTING_CYCLE, ERR_CLI_CYCLE_DELETE);
                        }
                    },
                    SyncOp::U => match wtr_svc.update_cycle_from_client(&cycle, time) {
                        Ok(_) => {
                            log_info!(INFO_CYCLE_UPD);
                            changed = true;
                        }
                        Err(err) => {
                            let erro = err.to_string();
                            log_error!(&erro);
                            msg_brkr.snd_error_to_clients(&erro, ERR_CLI_UPD_EXISTING_CYCLE);
                        }
                    },
                }
            }
            let sector_change = !data.sectors.is_empty();

            for sector_from_cli in &data.sectors {
                wtr_svc.update_physical_sector_from_client(sector_from_cli)
            }
            //this is outside the if bellow so it is in the lock block to avoid deadlock
            //probability of the cycles and/or cyclesectors access sectors info is greater than zero...i think :-)
            // REVIEW: porque a reconstrução dos setores pode influenciar coisas em curso no modo wizard
            // REVIEW: há que perceber em que modo está, e se as alterações em causa influenciam o estado de alguma forma.
            // REVIEW: como está está "ás cegas" e isso é "no good"
            if sector_change {
                // water_service.rebuild_cycles_sector_info();
                // com a alteração em 16/mai/2022, os ciclos a regar só são construidos no arranque do ciclo.
                // quer dizer que apenas se controla os ciclos, e os raw sectors em termos dos debitos, tempos maximos, precipitação, etc.
                // ainda assim temos que ver se está alguma coisa a correr para decidir o melhor curso de ação
                changed = true;
            }
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
        }
    }

    #[inline]
    pub fn process_alert(&self, alert: Alert, msg_broker: &MsgBrkr, time: CtrlTime) {
        let alert_msg = Alert::new_out(ExtMsgData::Alert(alert), time);
        msg_broker.snd_ext_msg(alert_msg);
    }

    #[inline]
    pub fn process_weather(&mut self, weather: Box<Weather>, msg_broker: &MsgBrkr, time: CtrlTime) {
        let weather_msg = ExtMsg::new_out(Topic::STC_WEATHER, ExtMsgData::Weather(weather), time);
        msg_broker.snd_ext_msg(weather_msg);
    }

    #[inline]
    pub fn process_changes(&mut self, time: CtrlTime, msg_brkr: &MsgBrkr, wtr_svc: &WtrSvc, wthr_svc: &WthrSvc, app_cfg: &mut AppCfg) {
        app_cfg.last_client_update = time;
        let actual_cfg: ClientContext;
        {
            let wtr_cfg = &wtr_svc.engine.wtr_cfg;
            actual_cfg = app_cfg.to_client(wtr_cfg, wthr_svc.get_context());
        }
        let prev_cfg: ClientContext = self.prev_cfg.clone();
        let db_sync = DBSync::build_partial(time, &prev_cfg, &actual_cfg, wtr_svc.cycles_clone_filtered(time), wtr_svc.sectors_clone_filtered(time));
        // se houver alguma alteração
        if !db_sync.sensors.is_empty() || !db_sync.cycles.is_empty() || !db_sync.sectors.is_empty() || db_sync.config.is_some() {
            if db_sync.config.is_some() {
                self.prev_cfg = prev_cfg;
            }
            let sync_msg = DBSync::new_out(db_sync, time);
            msg_brkr.snd_ext_msg(sync_msg);
        }
    }

    #[inline]
    fn send_full_db_to_client(&mut self, msg_brkr: &MsgBrkr, wtr_svc: &WtrSvc, wthr_svc: &WthrSvc, app_cfg: &mut AppCfg, time: CtrlTime) {
        app_cfg.last_client_update = time;
        let data: DBSync;
        {
            let wtr_cfg = &wtr_svc.engine.wtr_cfg;
            data = DBSync::build_full(time, &app_cfg.to_client(wtr_cfg, wthr_svc.get_context()), wtr_svc.cycles_clone(), wtr_svc.sectors_clone());
        }
        let full_db_msg = DBSync::new_out(data, time);
        msg_brkr.snd_ext_msg(full_db_msg);
    }
}
