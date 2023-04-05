use arrayvec::ArrayVec;

use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::watering_status::*;
use crate::services::irrigation::{db_model::*, sector_run::*, wtr_engine::*};
use crate::{log_info, logger::*};
use ctrl_prelude::{domain_types::*, globals::*, string_resources::*};

pub trait Stop {
    fn stop_man_watering_cycle(&mut self, o_cycle_id: Option<CYCLE_ID>, time: CtrlTime, watered_cycle_status: WateringStatus);
    fn stop_cycle(&mut self, time: CtrlTime, watered_cycle_status: WateringStatus);
    fn stop_sec(&mut self, time: CtrlTime);
}

impl Stop for WtrEngine {
    #[inline]
    fn stop_man_watering_cycle(&mut self, o_cycle_id: Option<CYCLE_ID>, time: CtrlTime, watered_cycle_status: WateringStatus) {
        let cycle_ptr: usize;
        let active_ptrs = &self.active_ptrs;
        if let Some(cycle_id) = o_cycle_id {
            // its assumed that one can stop any cycle at any time (assuming it is executing)
            // OTHER SPRINT when not a direct cycle, one could put the machine in manual mode automatically
            cycle_ptr = self.cycles.iter().position(|cycle| cycle.run.cycle_id == cycle_id).unwrap();
        } else if let Some(value) = active_ptrs.cycle {
            cycle_ptr = value as usize;
        } else {
            return;
        }
        // validate if the stopping cycle is realy the cycle instructed by the client to stop.  It's a guard as the backend do not control the client side
        //
        // Basically, ee test if the active cycle id is the id requested for stop.  
        // If it is not, we do not have nothing to do, as only the active cycle is running and we only run one cycle at a time
        let is_same_schedule = active_ptrs.cycle.unwrap() == cycle_ptr as u8;
        let have_active_cycle = active_ptrs.cycle.is_some(); //para agradar ao borrow checker

        if have_active_cycle && is_same_schedule {
            self.stop_cycle(time, watered_cycle_status);
        } else if !is_same_schedule {
            log_info!(warn_stop_inactive_cycle(&self.cycles[cycle_ptr].name));
        };
    }

    #[inline]
    // 1. when called in standrd mode, the sector is already stoped, and the cycle is terminating
    fn stop_cycle(&mut self, time: CtrlTime, watered_cycle_status: WateringStatus) {
        let cycle_ptr = self.active_ptrs.cycle.unwrap();
        if let Some(run_sector_ptr) = self.active_ptrs.run_sec_ptr {
            let run_sector = self.run_secs.get(run_sector_ptr as usize).unwrap();
            if run_sector.is_watering() {
                self.stop_sec(time);
            }
        }
        let cycle = &mut self.cycles[cycle_ptr as usize];
        // clean watering sectors still to execute, if any
        let mut not_watered_sectors: ArrayVec<SECTOR_PTR, MAX_SECTORS> = ArrayVec::new();
        let mut sector_run: &mut SectorRun;

        let db = &self.db;
        while let Some((sec_id, sec_ptr)) = self.wtr_secs.pop_front() {
            sector_run = &mut self.run_secs[sec_ptr as usize];
            // skipped reset
            sector_run.status = WateringStatus::Waiting;
            sector_run.skipped = true;
            sector_run.wtr_tgt_min = 0.0;
            sector_run.wtr_acc_min = 0.0;

            not_watered_sectors.push(sec_ptr);
            // update db with current run state
            db.upd_sec_run(sector_run).unwrap_or_else(|err| self.errors.push(Box::new(err)));
            log_info!(info_sector_not_watered(&self.sectors.get(sec_id as usize).unwrap().name));
        }
        upd_in_mem_cycle_on_end(cycle, watered_cycle_status, time);
        // update DB with next run info
        db.upd_cycle_run(&cycle.run).unwrap_or_else(|err| self.errors.push(Box::new(err)));

        // determine next schedule event
        cycle.schedule.retries_count += 1;
        if let Ok(Some(next_start)) = cycle.get_next_event(self.wtr_cfg.mode, cycle.run.end, &self.wtr_cfg.geo_pos) {
            cycle.schedule.start = next_start;          
        }
        // after machine start, obly in memory data counts
        cycle.run.start = cycle.schedule.start;
        cycle.run.run_id += 1;
        db.upd_cycle_srvr(cycle).unwrap_or_else(|err| self.errors.push(Box::new(err)));

        log_info!(info_cycle_end(&cycle.schedule.start.as_date_web_str_e(), &cycle.run.end.as_date_web_str_e()));
        self.active_ptrs.cycle = None;
    }

    #[inline]
    fn stop_sec(&mut self, time: CtrlTime) {
        let active_ptrs = &mut self.active_ptrs;
        let mut run_sector = &mut self.run_secs[active_ptrs.run_sec_ptr.unwrap() as usize];
        if run_sector.is_watering() {
            let sec = &mut self.sectors[run_sector.sec_id as usize];

            let valve_result = turn_off_sec(self.dev_svc.clone(), sec.device_id, sec.name.clone());
            upd_in_mem_run_sec(run_sector, valve_result.1, WateringStatus::Terminated, &mut self.errors);

            upd_in_mem_sec_on_stop(run_sector, sec, valve_result.0, time);
            upd_in_db_sec(&self.db, run_sector, sec, &mut self.errors);
        }
        run_sector.is_manual = false;
        active_ptrs.reset_sec();
    }
}
