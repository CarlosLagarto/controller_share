// DESIGN DECISION
//
// tContext information should be passed as a parameter on thread start.
// The original idea iof having a dynamic config seems to be overly complicated.
// If reconfig is needed, just restart the service.
// An alternative would be program an automatic restart to refresh the config, but even so, overly complicated.
//
//     The ScheduledCycle table only have the active schedules.
//     It can be more than one active schedule. Either by user definition (standard) or by wizard operation.
//     Historic is preserved in the WateredCycle and WateredSector tables, plus current logs..
//     It can be defined more than one schedule.
//     Deleteions are ok.
//
//     Wizard is automatically defined on start if one is not already defined.
//
//     Other situation have to be managed by client UI.
//     Arranque
//           standard - reads db and if one is not defined stays in no-schedule def mode
//           wizard - reads db and if one is not defined, creates one
//
//     Some exceptions to consider:
//         - status read does not change state
//         - client updates - one should put the machine in manual and then agina in the desired mode.
//
// Run control sends a tick each second to the water machine for the commands/events execution that may be waiting in queue.
//
// Base principle design, all BD is loaded in memory.
//
// Regarding the cycles, read above.  Regarding the sectors, only on sector is active at any given moment, by design.
// Other systems may have the ability to sustain watering several sectors simulstaneously.  
// In home systems that seems to me to have a low probability - and my pump don't have that power, so....this reflects my requirements
//
// This decision simplifies logic, as we only have to check one active sector and operate exclusively that sector, 
// reducing the nr of validatuin cycles 

use std::sync::Arc;

use crate::app_time::{ctrl_time::*, schedule::*};
use crate::data_structs::client::sector_cli::*;
use crate::data_structs::msgs::{alert::*, alert_thresholds::*, weather::*};
use crate::data_structs::rega::{command::*, mode::Mode, state::*};
use crate::db::db_sql_lite::*;
use crate::services::electronics::devices_svc::*;
use crate::services::irrigation::{actions::exec_cmds::*, errors::cycle::*};
use crate::services::irrigation::{cycle::*, db_model::*, sector::*, sector_run::*, stress_ctrl::*, wtr_engine::*};
use crate::services::msg_broker::msg_brkr_svc::*;
use crate::{log_info, logger::*};
use arrayvec::ArrayVec;
use ctrl_prelude::{domain_types::*, globals::*, string_resources::*};

pub struct WtrSvc {
    pub engine: WtrEngine,
}

impl WtrSvc {
    #[inline]
    pub fn new(alert_thresholds: AlrtThresholds, msg_brkr: SMsgBrkr, db: Persist, start_up_time: CtrlTime, devices: Arc<DevicesSvc>) -> Self {
        let mut ws = Self {
            engine: WtrEngine::build(alert_thresholds, msg_brkr, db, start_up_time, devices),
        };
        ws.engine.snd_command(Command::Start);
        ws.engine.exec_cmmnds(start_up_time);
        ws
    }

    #[inline]
    pub fn verify_things_to_do(&mut self, time: CtrlTime) {
        let engine = &mut self.engine;

        // very stress validation schedule
        engine.stress_process_time_tick(time);
        // verify cycles and sectors timmings
        engine.process_time_tick(time);

        engine.exec_cmmnds(time);
    }

    #[inline]
    pub fn process_alert(&mut self, alert: Alert) {
        let alert = alert;
        let engine = &mut self.engine;
        if alert.type_.is_rain_or_wind(){
            engine.wtr_cfg.in_alert = alert.type_ as u8;
            log_info!(info_whtr_alert_rcvd(&alert.type_.to_string()));
            engine.snd_command(Command::Suspend(alert));           
        }
    }

    #[inline]
    pub fn process_weather(&mut self, weather: &Weather) {
        let engine = &mut self.engine;
        if engine.suspend_timeout.is_some() {
            // one only needs to process time events if the machine is in the suspend state
            if !engine.alrt_thresholds.is_weather_alert(weather.rain_period, weather.wind_intensity) {
                // and if is bellow alert thresholds.
                engine.wtr_cfg.in_alert = AlertType::NoAlert as u8;
                engine.snd_command(Command::Resume);
            }
        }
    }

    /// At this point, the cycle is already in the database
    ///
    /// Force the water machine to relaunch all cycle and associated time controls
    ///
    /// Change mode is used to, in this case, switch to the same mode, but reexecute the internal states to accomplish this.
    ///
    #[inline]
    pub fn process_cycle_added(&mut self) {
        self.engine.snd_command(Command::ChangeMode(self.engine.wtr_cfg.mode));
    }

    #[inline]
    pub fn process_command(&mut self, cmd: Command, time: CtrlTime) {
        let engine = &mut self.engine;
        log_info!(info_wtr_eng_cmd_rcvd(&cmd.to_string()));
        engine.snd_command(cmd);
        engine.exec_cmmnds(time);
    }

    #[inline]
    pub fn save_physical_sector_list(&self) {
        _ = self.engine.save_secs(); //save_secs already logs the error
    }

    #[inline]
    pub fn add_cycle_from_client(&mut self, cycle_cli: Cycle, time_next_event: CtrlTime) {
        let engine = &mut self.engine;
        let mut cycle_cli = cycle_cli;
        cycle_cli.schedule.start = time_next_event;
        cycle_cli.run.run_id = 0; //info from client which we force the initialization, just in case.
        if let Ok(Some(ptr)) = engine.add_cycle(cycle_cli, None) {
            // just ignore potential erros that will be already registered inside the called function
            // and the internal standard cycles list is updated
            self.engine.std_ptrs.push((self.engine.std_ptrs.len() as u32, ptr));
        };
    }

    #[inline]
    pub fn delete_cycle_from_client(&mut self, cycle_id: CYCLE_ID) -> CycleResult<()> {
        self.engine.del_cycle_from_id(cycle_id).map_or_else(
            |_e| Err(CycleError::CantDeleteCycleSchedule(cycle_id.to_string())),
            |ptr| {
                let pos = self.engine.std_ptrs.iter().position(|t| t.1 == ptr);
                if let Some(idx) = pos {
                    self.engine.std_ptrs.swap_remove(idx);
                } else {
                    unreachable!(); // if code arrives here, something wrong eists in the pointers/indexes
                }
                Ok(())
            },
        )
    }

    #[inline]
    pub fn update_cycle_from_client(&mut self, cycle_cli: &Cycle, time: CtrlTime) -> CycleResult<()> {
        let cli_cycle_id = cycle_cli.run.cycle_id;
        let engine = &mut self.engine;
        let cycle = engine.get_std_cycle_by_id_mut(cli_cycle_id);
        let cycle_ptr = cycle.ptr.unwrap();
        cycle.update_with_cli_info(cycle_cli);

        match find_next_event(time, &cycle.schedule) {
            Ok(Some(start)) => {
                cycle.schedule.start = start;
                let srv_cycle = &mut engine.cycles[cycle_ptr as usize];
                engine.db.upd_cycle_srvr(srv_cycle).map(|_| ()).map_err(|_err| CycleError::CantUpdateCycleSchedule(srv_cycle.name.clone()))
            }
            Ok(_) => Ok(()), //nop
            Err(err) => Err(CycleError::CantUpdateCycleSchedule(err.to_string())),
        }
    }

    #[inline]
    pub fn update_physical_sector_from_client(&mut self, sector_from_cli: &SectorCli) {
        let sec = &mut self.engine.sectors[sector_from_cli.id as usize];
        sec.update_with_cli_info(sector_from_cli);
    }

    /// The general idea is to send commands to the queue that will be checked and executed every second.
    /// The exception is the termination, where we want to finish asap, and so the exxec_commands is also called here
    #[inline]
    pub fn terminate(&mut self, time: CtrlTime) {
        let engine = &mut self.engine;
        engine.snd_command(Command::ShutDown);
        engine.exec_cmmnds(time);
    }

    #[inline]
    #[rustfmt::skip]
    pub const fn state(&self) -> State { self.engine.wtr_cfg.state }

    #[inline]
    #[rustfmt::skip]
    pub const fn get_mode(&self) -> Mode { self.engine.wtr_cfg.mode }

    #[inline]
    #[rustfmt::skip]
    pub fn snd_command(&mut self, cmd: Command) { self.engine.snd_command(cmd); }

    #[inline]
    #[rustfmt::skip]
    pub fn cycles_clone(&self) -> CycleList { self.engine.cycles.clone() }

    #[inline]
    pub fn cycles_clone_filtered(&self, last_sync: CtrlTime) -> CycleList {
        let mut cycle_list: CycleList = ArrayVec::<Cycle, MAX_CYCLES>::new();
        for cycle in self.engine.cycles.iter() {
            if cycle.last_change > last_sync {
                cycle_list.push(cycle.clone());
            }
        }
        cycle_list
    }

    #[inline]
    pub fn sectors_clone_filtered(&self, last_sync: CtrlTime) -> SecList {
        let mut sec_list: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
        for sec in self.engine.sectors.iter() {
            if sec.last_change > last_sync {
                sec_list.push(sec.clone());
            }
        }
        sec_list
    }

    #[inline]
    pub fn run_sectors_clone_filtered(&self, last_sync: CtrlTime, filtered_sectors: &SecList) -> SecRunList {
        let mut sec_run_list: SecRunList = ArrayVec::<SectorRun, MAX_SECTORS>::new();
        let len_run_secs = self.engine.run_secs.len();
        if len_run_secs > 0 {
            for sec in filtered_sectors.iter() {
                if sec.last_change > last_sync && (sec.id as usize) < len_run_secs {
                    sec_run_list.push(self.engine.run_secs[sec.id as usize].clone());
                }
            }
        }
        sec_run_list
    }
}
