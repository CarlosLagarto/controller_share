use arrayvec::ArrayVec;
use std::collections::VecDeque;
use std::{error::Error, ops::Add, sync::Arc};

use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*, tm::*};
use crate::config::wtr_cfg::*;
use crate::data_structs::msgs::alert_thresholds::*;
use crate::data_structs::rega::{command::*, internal::*, mode::*, running_ptr::*, state::*, watering_status::*};
use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::irrigation::{actions::stop::*, cycle::*, db_model::*, errors::sector::*, sector::*, sector_run::*, wzrd_algorithms::*};
use crate::services::{electronics::devices_svc::*, electronics::valve_state::*, msg_broker::msg_brkr_svc::*};
use crate::{log_info, log_warn, logger::*};
use ctrl_prelude::{domain_types::*, globals::*, string_resources::*};

const MAX_ERRORS: usize = 5;
const MAX_CMDS: usize = 5;

pub type SectorTuple = (SECTOR_ID, SECTOR_PTR);
pub type CycleTuple = (CYCLE_ID, CYCLE_PTR);

pub type SecList = ArrayVec<Sector, MAX_SECTORS>;
pub type SecRunList = ArrayVec<SectorRun, MAX_SECTORS>;
pub type CycleList = ArrayVec<Cycle, MAX_CYCLES>;
/// Dimension 28
pub type PtrList = ArrayVec<CycleTuple, MAX_STANDARD_CYCLES>;
pub type WtrSecList = VecDeque<SectorTuple>;
pub type InsertResult = Result<Option<CYCLE_PTR>, DBError>;

/// Dimension = 1784
pub struct WtrEngine {
    pub cycles: CycleList, //24 + 3 ou 2 * 312 = 624 ou 936
    pub sectors: SecList,
    pub dev_svc: Arc<DevicesSvc>,
    pub run_secs: SecRunList, //heap = 6 * (64) = 384
    /// tupple vetor,.  0 = sector id; 1 = run sector ptr
    pub wtr_secs: WtrSecList, //heap = 16 * 6 = 128
    pub wtr_cfg: WtrCfg,
    pub cmd_queue: VecDeque<Command>,
    pub msg_broker: SMsgBrkr,
    pub db: Persist,
    pub active_ptrs: RunningPtr,
    // it aids to speedup cycles seacrh - in memory - for the designed sceneries
    // Usually we will have 3 active cycles in memory, (although the definition of standard cycles is not limited):
    //      A- Wizard Cycle automatically created
    //      B- Support Cycle for the direct cycle or direct sector
    //      C- Standard cycles
    // internal holds A and B types
    pub internal: InternalPtr,
    // std_ptrs holds C type
    /// tuple vector,.  0 = cycle id; 1 = cycle ptr
    pub std_ptrs: PtrList,
    //suspend control
    pub suspend_timeout: Option<CtrlTime>,
    // other
    pub alrt_thresholds: AlrtThresholds,
    pub stress_control_sched: Schedule,
    pub errors: Vec<Box<dyn std::error::Error>>,
}

impl WtrEngine {
    #[inline]
    pub fn build(alrt_thresholds: AlrtThresholds, msg_broker: SMsgBrkr, db: Persist, start_up_time: CtrlTime, dev_svc: Arc<DevicesSvc>) -> Self {
        log_info!(INFO_STARTING_STATE_MACHINE);

        let mut wtr_cfg = WtrCfg::new(db.clone(), start_up_time);
        wtr_cfg.state = State::Starting;
        let stress_control_interval_secs: u16 = min_to_sec_f32(wtr_cfg.wizard_info.stress_control_interval as f32) as u16;
        let mut sectors: SecList = ArrayVec::<Sector, MAX_SECTORS>::new();
        db.get_cfg_secs(&mut sectors).unwrap();

        // start a litle after start-of-day, of the next day, so not to colide with end-of-day processes
        let mut stress_start_date = start_up_time.as_utc_date_time_e();
        stress_start_date.sec = 10;
        stress_start_date.nanos = 0;
        stress_start_date.min += 1;

        let stress_start = CtrlTime::from_utc_date_time_e(&stress_start_date);

        let stress_control_sched = Schedule::build_run_forever(stress_start, stress_control_interval_secs, ScheduleRepeatUnit::Days);

        let cmd_queue = VecDeque::with_capacity(MAX_CMDS);
        let mut engine = WtrEngine {
            wtr_cfg,
            cycles: ArrayVec::<Cycle, MAX_CYCLES>::new(),
            sectors,
            dev_svc,
            run_secs: ArrayVec::<SectorRun, MAX_SECTORS>::new(),
            wtr_secs: VecDeque::with_capacity(MAX_SECTORS),
            active_ptrs: RunningPtr::default(),
            suspend_timeout: None,
            cmd_queue,
            msg_broker,
            db,
            alrt_thresholds,
            stress_control_sched,
            errors: Vec::with_capacity(MAX_ERRORS), // Its supposed to be empty, unless...well errors
            internal: InternalPtr::default(),
            std_ptrs: ArrayVec::<CycleTuple, MAX_STANDARD_CYCLES>::new(),
        };

        let dev_svc = &engine.dev_svc;
        // verify all sectors on start to see if they are online and query the status
        // if not online, we have an error and the sector is disabled
        // if they are online, but we can't change the state (usuallly means that they are offline), the sector is also disabled
        let mut valve_status;
        for sector in engine.sectors.iter_mut() {
            valve_status = dev_svc.inner.read().relay_status(sector.device_id);
            match valve_status {
                RelayState::Error => {
                    sector.enabled = false;
                    error!("A valvula do sector: {} não está acessivel.  shelly inacessivel?", &sector.name);
                }
                RelayState::Open => {
                    if dev_svc.inner.read().turn_off_physical_sec(sector.device_id) {
                        log_warn!(warn_wtr_adptr_close_valve_on_init(&sector.name));
                    } else {
                        // In thesis, this never happens, as we just queried the valve status and it was live
                        // nevertheless, we leave this here for logic completion.  Strange things happen sometimes :-)
                        sector.enabled = false;
                        error!("A valvula do sector: {} não está acessivel.  shelly inacessivel?", &sector.name);
                    }
                }
                _ => (),
            }
        }
        engine
    }

    #[inline]
    /// As we only have one active sector by design, we only need to stop that sector, if active.
    pub fn process_interrupt(&mut self, time: CtrlTime, watered_cycle_status: WateringStatus) {
        if self.active_ptrs.cycle.is_some() {
            self.stop_cycle(time, watered_cycle_status);
        }
    }

    /// Returns NextState, if current state should be interrupted, or the calculated next water state
    #[inline]
    pub fn is_interrupt_and_change_command(&self, cmd: &Command) -> (State, bool, WateringStatus) {
        match *cmd {
            Command::ChangeMode(_) => (State::EstablishMode, true, WateringStatus::Terminated),
            Command::ShutDown => (State::Shutdown, true, WateringStatus::Terminated),
            Command::Error => (State::Error, true, WateringStatus::Error),
            _ => {
                if self.active_ptrs.cycle.is_some() {
                    (self.wtr_cfg.state, false, WateringStatus::Running)
                } else {
                    (self.wtr_cfg.state, false, WateringStatus::Waiting)
                }
            }
        }
    }

    #[inline]
    pub fn is_new_cycle(&self, cycle_id: CYCLE_ID) -> bool {
        let pos = self.cycles.iter().position(|cycle| cycle.run.cycle_id == cycle_id);
        pos.is_none()
    }

    #[inline]
    pub fn snd_command(&mut self, cmd: Command) {
        self.cmd_queue.push_back(cmd);
    }

    #[inline]
    #[rustfmt::skip]
    pub fn validate_errors(&self, state: State) -> State {
        if !self.errors.is_empty() { State::Error } else { state }
    }

    #[inline]
    pub fn save_secs(&self) -> SimpleResult {
        self.db.upd_secs_batch(&self.sectors)
    }

    #[inline]
    pub fn upd_cycle_and_ins_secs(&mut self, cycle: &mut Cycle, errors: &mut Vec<Box<dyn Error>>) {
        self.db.upd_cycle_srvr(cycle).unwrap_or_else(|e| errors.push(Box::new(e)));
        self.db.ins_cycle_run(&cycle.run).unwrap_or_else(|e| errors.push(Box::new(e)));
        for (_, sec_ptr) in self.wtr_secs.iter() {
            self.db.ins_secs_run(&self.run_secs[*sec_ptr as usize]).unwrap_or_else(|e| errors.push(Box::new(e)));
        }
    }

    #[inline]
    pub fn get_std_cycle_by_id_mut(&mut self, id: CYCLE_ID) -> &mut Cycle {
        // O(n), but size will be 1 to 3 elements,  99,999% , in normal operation
        let pos = self.std_ptrs.iter().position(|ptr| ptr.0 == id).unwrap();
        self.cycles.get_mut(self.std_ptrs[pos].1 as usize).unwrap()
    }

    #[inline]
    pub fn get_std_cycle_by_id(&mut self, id: CYCLE_ID) -> &Cycle {
        // O(n), but size will be 1 to 3 elements,  99,999% , in normal operation
        let pos = self.std_ptrs.iter().position(|ptr| ptr.0 == id).unwrap();
        self.cycles.get(self.std_ptrs[pos].1 as usize).unwrap()
    }

    /// Adds a cycle to the machine
    /// Returns cycle ptr, or an error, that despite beeing logged here, is used for some logic on the client side.
    #[inline]
    pub fn add_cycle(&mut self, mut cycle: Cycle, new_pos: Option<usize>) -> Result<Option<CYCLE_PTR>, DBError> {
        let res = self.db.ins_cycle(&mut cycle);
        match res {
            Ok(_) => {
                let cycle_ptr = if let Some(ptr) = new_pos {
                    self.cycles.insert(ptr, cycle);
                    ptr
                } else {
                    let result = self.cycles.binary_search_by_key(&cycle.cycle_type, |v| v.cycle_type);
                    let ptr = match result {
                        Ok(ptr) => ptr,
                        Err(ptr) => ptr,
                    };
                    self.cycles.insert(ptr, cycle);
                    ptr
                };
                Ok(Some(cycle_ptr as u8))
            }
            Err(e) => Err(e),
        }
    }

    /// Makes the setup of the wizard cycles
    ///
    /// On running condition. "get next event" updates the cycle for the next running moment.
    ///
    #[inline]
    pub fn set_wizard_cycle(&mut self, time: CtrlTime, new_pos: Option<usize>) -> Result<Option<CYCLE_PTR>, DBError> {
        let cycle = Cycle::new_wizard(time, self.wtr_cfg.geo_pos);
        self.add_cycle(cycle, new_pos)
    }

    #[inline]
    pub fn set_direct_cycle(&mut self, time: CtrlTime, new_pos: Option<usize>) -> Result<Option<CYCLE_PTR>, DBError> {
        let cycle = Cycle::new_direct(time);
        self.add_cycle(cycle, new_pos)
    }

    /// Removes the cycle from the DB
    #[inline]
    pub fn del_cycle_from_id(&mut self, cycle_id: CYCLE_ID) -> DBResult<CYCLE_PTR> {
        let ptr: CYCLE_PTR;
        {
            let cycle = self.get_std_cycle_by_id(cycle_id);
            ptr = cycle.ptr.unwrap();
        }
        self.cycles.remove(ptr as usize);
        self.db.del_cycle_by_id(cycle_id).map(|_| ptr)
    }
}

#[rustfmt::skip]
#[inline]
pub fn turn_off_sec(dev_svc: SDevicesSvc, device_id: DEVICE_ID, sec_name: String) -> (RelayState, Option<SecErr>) {
    if dev_svc.inner.read().turn_off_physical_sec(device_id) {
        (RelayState::Closed, None)
    } else {
        (RelayState::Error, Some(SecErr::UnknownErrOpenValve { sec_name, } ))
    }
}

#[inline]
#[rustfmt::skip]
pub fn turn_on_sec(dev_svc: SDevicesSvc, device_id: DEVICE_ID, sec_name: String) -> (RelayState, Option<SecErr>) {
    if dev_svc.inner.read().turn_on_physical_sec(device_id) { 
        (RelayState::Open, None)
    } else {
        (RelayState::Error, Some(SecErr::UnknownErrOpenValve { sec_name, } ))
    }
}

#[inline]
pub fn upd_in_db_sec(db: &Persist, run_sector: &SectorRun, sec: &Sector, errors: &mut Vec<Box<dyn Error>>) {
    db.upd_sec_run(run_sector).unwrap_or_else(|e| errors.push(Box::new(e)));
    db.upd_sec(sec).unwrap_or_else(|e| errors.push(Box::new(e)));
}

#[inline]
pub fn upd_in_db_cycle(db: &Persist, cycle: &Cycle, errors: &mut Vec<Box<dyn Error>>) {
    db.upd_cycle_run(&cycle.run).unwrap_or_else(|e| errors.push(Box::new(e)));
    db.upd_cycle_srvr(cycle).unwrap_or_else(|e| errors.push(Box::new(e)));
}

#[inline]
#[rustfmt::skip]
pub fn upd_in_mem_run_sec(run_sec: &mut SectorRun, error: Option<SecErr>, status_ok: WateringStatus, errors: &mut Vec<Box<dyn Error>>) {
    if let Some(e) = error {
        run_sec.status = WateringStatus::Error;
        push_error(run_sec, errors, e);
    } else {
        run_sec.status = status_ok;
    }
}

#[inline]
pub fn push_error(sec_run: &mut SectorRun, errors: &mut Vec<Box<dyn Error>>, err: SecErr) {
    sec_run.status = WateringStatus::Error;
    errors.push(Box::new(err));
}

#[inline]
pub fn upd_in_mem_sec_on_stop(run_sector: &mut SectorRun, sec: &mut Sector, valve_state: RelayState, time: CtrlTime) {
    run_sector.end = time;
    sec.state = valve_state;
    // deficit is updated in every interruption
    // The idea is that one can have several suspended states during watering.
    //
    // so each time it stops we udate the minutes in wtr_acc_min
    run_sector.wtr_acc_min += nano_to_min(time.0 - run_sector.last_start.0);
    // and update the deficit with the remaining time
    // in the subsequent start, the deficit is updated with this value and will be considered in the remaining cycle time
    sec.deficit = (sec.deficit - (run_sector.wtr_acc_min * sec.debit)).min(-1.); // <0 => run off

    disable_if_not_ok(sec);

    sec.last_change = time;
    sec.last_watered_in = time;
}

#[inline]
pub fn upd_in_mem_cycle_on_end(cycle: &mut Cycle, new_status: WateringStatus, time: CtrlTime) {
    let mut cycle = cycle;
    cycle.run.end = time;
    cycle.last_run = cycle.schedule.start;
    cycle.run.status = new_status;
    cycle.last_change = time;
}

#[inline]
pub fn cfg_run_secs(sectors: &SecList, run_sec_list: &mut SecRunList, cycle_id: CYCLE_ID, run_id: CYCLE_RUN) {
    for sec in sectors.iter().filter(|sec| sec.enabled) {
        run_sec_list.push(SectorRun::new(cycle_id, run_id, sec.id, WateringStatus::Waiting));
    }
}

#[inline]
pub fn upd_run_sec_data(
    run_sec: &mut SectorRun, sec_start: CtrlTime, cycle_id: CYCLE_ID, run_id: CYCLE_RUN, duration_minutes: f32, pump_recycle_secs: f32,
) -> CtrlTime {
    run_sec.start = sec_start;
    run_sec.cycle_id = cycle_id;
    run_sec.curr_run = run_id;
    run_sec.wtr_tgt_min = duration_minutes;
    run_sec.end = sec_start.add(min_to_nano(run_sec.wtr_tgt_min));
    // add some _pump_recycle_time to ease on the water pump
    run_sec.end.add_secs_f32(pump_recycle_secs)
}

/// Return (minutes, deficit)
#[inline]
pub fn calc_dur_and_deficit(wtr_cfg: &WtrCfg, run_sec: &SectorRun, db: &Persist, sec: &Sector, sec_start: CtrlTime) -> (f32, f32) {
    match wtr_cfg.mode {
        Mode::Manual => dur_man_strategy(run_sec.is_manual, db, sec, wtr_cfg, sec_start),
        Mode::Standard => dur_std_strategy(db, sec, wtr_cfg, sec_start),
        Mode::Wizard => dur_wzrd_strategy(db, sec, wtr_cfg, sec_start),
    }
}

#[inline]
pub fn disable_if_not_ok(sec: &mut Sector) {
    if sec.state == RelayState::Error {
        //disable the sector until we understand what may have went wrong.
        // have to be enabled manually aftwerwards
        // REVIEW - maybe send an alert to catch the atention for this in the client UI
        sec.enabled = false;
    }
}

#[inline]
pub fn update_in_db_all(db: &Persist, sector_run: &SectorRun, sec: &Sector, cycle: &Cycle, errors: &mut Vec<Box<dyn Error>>) {
    upd_in_db_sec(db, sector_run, sec, errors);
    upd_in_db_cycle(db, cycle, errors);
}
