use crate::app_time::{ctrl_time::*, tm::min_to_nano};
use crate::data_structs::rega::{running_ptr::*, watering_status::*};
use crate::services::irrigation::{sector::*, sector_run::*, wtr_engine::*, wzrd_algorithms::*};
use crate::{log_info, logger::*};
use ctrl_prelude::string_resources::*;

// On weather alert event, if machine is watering, changes to suspend state
// if actual weather dimninihes bellow alert threshold, or max time passes (40' configured at the moment)
// or timeout and interrupt watering cycle rescheduling the next watering event.
// REVIEW - it may be necessary to adjust cycle for two times a day, in which case we will have to change the schedule to 24 hours,
// so it can be adjusted to 12 hours - which have implication in the sunrise and sunset logic, 1 hour later perhaps.
// Have also to review the logic for sevral alerts in a row, which as scenario that may happen
// Have to keep track of the acc stoped time and not only the schedule, because with intermitent stops, 
// its better to stop alltogether and wait for the rain to stop
pub trait Suspend {
    fn suspend(&mut self, time: CtrlTime);
    fn resume(&mut self, time: CtrlTime) -> bool;
}

impl Suspend for WtrEngine {
    ///Only called during an active cycle 
    #[inline]
    fn suspend(&mut self, time: CtrlTime) {
        if let RunningPtr {
            cycle: Some(cycle_ptr),
            sec_id: Some(sec_id),
            run_sec_ptr: Some(run_sec_ptr),
        } = self.active_ptrs
        {
            let cycle = &mut self.cycles[cycle_ptr as usize];
            let run_sec = self.run_secs.get_mut(run_sec_ptr as usize).unwrap();
            let sec = &mut self.sectors[sec_id as usize];

            let valve_result = turn_off_sec(self.dev_svc.clone(), sec.device_id, sec.name.clone());
            if valve_result.1.is_some(){
                sec.enabled = false;
                info!("Water machine not suspended at: {} due to valve error.", time.as_rfc3339_str_e());
            }else{
                info!("Water machine suspended at: {}", time.as_rfc3339_str_e());
            }

            upd_in_mem_run_sec(run_sec, valve_result.1, WateringStatus::Suspended, &mut self.errors);
            // adjust sector end time to actual value + max suspension config value
            upd_in_mem_sec_on_stop(run_sec, sec, valve_result.0, time);
            upd_in_mem_cycle_on_end(cycle, WateringStatus::Suspended, time);

            update_in_db_all(&self.db, run_sec, sec, cycle, &mut self.errors);

            self.suspend_timeout = Some(time + min_to_nano(self.wtr_cfg.wizard_info.suspend_timeout.into()));

            // difference between suspend and stop sec states, is that we do not reset the active sec on restart during resume
            // and put back again the sector on que watering queue
            self.wtr_secs.push_front((sec_id, run_sec_ptr));
            log_info!(info_cycle_end(&cycle.schedule.start.as_date_web_str_e(), &cycle.run.end.as_date_web_str_e()));
        } else {
            self.suspend_timeout = Some(time + min_to_nano(self.wtr_cfg.wizard_info.suspend_timeout.into()));
        }
        
    }

    // if there is an active cycle, restart it , returning true
    // otherwise returns false, notidying the caller that the cycle shoud be terminated
    // Question is, there is only a suspended state with an active cycle and sector
    // but we need  if let to recover in one shot all running running ptr variables, i.e., struct destructure
    #[inline]
    fn resume(&mut self, time: CtrlTime) -> bool {
        let mut result = false;
        if let RunningPtr {
            cycle: Some(cycle_ptr),
            sec_id: Some(sec_id),
            run_sec_ptr: Some(run_sec_ptr),
        } = self.active_ptrs
        {
            let db = &self.db;

            //if it is suspended & active_cycle isn't valid, something wrong happend before
            let cycle = &mut self.cycles[cycle_ptr as usize];
            let mut sector_run = &mut self.run_secs[run_sec_ptr as usize];

            // find sector start time
            let time_remaining = sector_run.wtr_tgt_min - sector_run.wtr_acc_min;

            // also evaluates all the start and end times of the remaining secs
            // handle/update sector resume data
            sector_run.last_start = time; // first setor do not need pump recycle time
            sector_run.end = time + min_to_nano(time_remaining);
            sector_run.status = WateringStatus::Waiting;

            // adjust queued/pending sectors start times including the suspended one that we pushed back into the queue in the suspend action
            let wtr_cfg = &self.wtr_cfg;
            let secs: &mut SecList = &mut self.sectors;
            let wtr_secs: &mut WtrSecList = &mut self.wtr_secs;
            let mut sec_start = time;
            let pump_recycle_secs: f32 = wtr_cfg.pump_recycle_time as f32;
            let mut sec: &mut Sector;
            let mut run_sec: &mut SectorRun;

            let mut duration_minutes: f32;
            let mut new_deficit;

            for (sec_id, run_sec_ptr) in wtr_secs {
                sec = &mut secs[*sec_id as usize];
                run_sec = &mut self.run_secs[*run_sec_ptr as usize];
                // suspend state only applies to wizard mode
                // in the resume state, eT was already calculated, so, we only need to check if there was any rain in the mean time
                (duration_minutes, new_deficit) = dur_wzrd_strategy_for_resume(sec, wtr_cfg, sec_start, cycle.schedule.start);
                // Wizard mode don't water if less than for 1 minute, that would not have any practical effect, saving wtr engine, water, energy, maintenance costs, ...
                // Any calculated value is already in the new_deficit value calculated above.
                if duration_minutes >= 1. {
                    sec_start = upd_run_sec_data(run_sec, sec_start, cycle.run.cycle_id, cycle.run.run_id, duration_minutes, pump_recycle_secs);
                }
                sec.deficit = new_deficit;
            }

            sec = &mut self.sectors[sec_id as usize];
            sec.last_change = time;

            cycle.run.end = CtrlTime(0);
            cycle.last_change = time;
            cycle.run.status = WateringStatus::Running;

            // and now  start the sector, or end cycle, depending on having sectors to run or not...
            // It seems that there may exists an edge situation, as the suspend only can hapen when watering, so in principle we always interrupt a watering sector...
            // but this may happen in the last second or so, so in theory, we can suspend at the same time that the sector will end watering....
            // TODO test this edge case one of these days.
            if !self.wtr_secs.is_empty() {
                let (sec_id, run_sec_ptr) = self.wtr_secs.pop_front().unwrap();
                run_sec = &mut self.run_secs[run_sec_ptr as usize];
                let valve_result = turn_on_sec(self.dev_svc.clone(), sec.device_id, sec.name.clone());
                if valve_result.1.is_some(){
                    sec.enabled = false;
                    info!("Water machine not resumed at: {} due to valve error.", time.as_rfc3339_str_e());
                }else{
                    result = true;
                    info!("Water machine resumed at: {}", time.as_rfc3339_str_e());
                }
                sec.state = valve_result.0;
                upd_in_mem_run_sec(run_sec, valve_result.1, WateringStatus::Running, &mut self.errors);

                self.active_ptrs.sec_id = Some(sec_id);
                self.active_ptrs.run_sec_ptr = Some(run_sec_ptr);
            }
            update_in_db_all(db, &self.run_secs[run_sec_ptr as usize], sec, cycle, &mut self.errors);
        }
        result
    }
}
