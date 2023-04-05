use crate::app_time::{ctrl_time::*, tm::*};
use crate::data_structs::rega::watering_status::*;
use crate::services::irrigation::{db_model::*, sector::*, sector_run::*, wtr_engine::*};
use crate::{log_info, log_warn, logger::*};
use ctrl_prelude::{domain_types::*, string_resources::*};

pub trait Start {
    fn start_cycle_not_manual(&mut self, cycle_ptr: CYCLE_PTR);
    fn start_cycle(&mut self, is_manual: bool, time: Option<CtrlTime>);
    fn start_sec(&mut self, is_manual: bool);
}

impl Start for WtrEngine {
    #[inline]
    fn start_cycle_not_manual(&mut self, cycle_ptr: CYCLE_PTR) {
        let active_ptrs = &mut self.active_ptrs;
        if active_ptrs.cycle.is_none() {
            active_ptrs.cycle = Some(cycle_ptr);
            self.start_cycle(false, None);
        } else {
            active_ptrs.cycle = None;
            log_warn!(WARN_WATER_CYCLE_CONFLICT);
        }
    }

    #[inline]
    fn start_cycle(&mut self, is_manual: bool, time: Option<CtrlTime>) {
        let db = &self.db;
        let wtr_secs: &mut WtrSecList = &mut self.wtr_secs;

        let cycle_ptr = self.active_ptrs.cycle.unwrap();

        let cycle = &mut self.cycles[cycle_ptr as usize];
        log_info!(info_cycle_time(&cycle.name, &cycle.schedule.start.as_date_web_str_e()));

        // on cycle start config the sectors to water
        self.run_secs.clear();
        wtr_secs.clear();

        let cycle_start_time: CtrlTime;
        if let Some(time_) = time{
            cycle_start_time = time_;
            cycle.schedule.start = cycle_start_time;
        }else{
            cycle_start_time = cycle.schedule.start;
        }
        // always increment the cycle run - necessary to avoid colision in db in the watered cycles and sectors tables
        cycle.run.run_id += 1; 
        cycle.run.start = cycle_start_time;

        // always reconfig the sectors, except on force sector, where the active sector ir pre-prepared with needed data
        if !is_manual || self.active_ptrs.sec_id.is_none() {
            // only matter the enabled sectors
            cfg_run_secs(&self.sectors, &mut self.run_secs, cycle.run.cycle_id, cycle.run.run_id);
        } else {
            // if is manual mode or forced cycle/sector
            let sector_id = self.active_ptrs.sec_id.unwrap();
            // reconfig for forced sector is specific as it have only one sector
            let mut run_sec = SectorRun::new(cycle.run.cycle_id, cycle.run.run_id, sector_id, WateringStatus::Waiting);
            run_sec.is_manual = true;
            self.run_secs.push(run_sec);
            // sector ptr in sector direct is always 0
            wtr_secs.push_back((sector_id, 0));
        }

        
        cycle.run.status = WateringStatus::Running;
        cycle.run.end = CtrlTime(0); // clean end, as we do not know when it will end
        cycle.last_change = cycle_start_time; // consider last change as the cycle start time

        let wtr_cfg = &self.wtr_cfg;
        let mut sec_start = cycle_start_time;
        let pump_recycle_secs: f32 = wtr_cfg.pump_recycle_time as f32;
        let mut sec: &mut Sector;

        let mut duration_minutes: f32;
        let mut new_deficit: f32;

        // update data
        // sector duration depends on the mode
        // update cycle id and run
        // only in wizard mode we have dynamic duration, different from the ones defined in the sectors
        // So, we determine the duration as a function of the mode, and the remaining actions depend on the duration
        // if we skip a sector, the next start will be the same, meaning, time does not advance
        for (run_sec_ptr, run_sec) in self.run_secs.iter_mut().enumerate() {
            sec = &mut self.sectors[run_sec.sec_id as usize];
            // initial acc configuration is zero
            run_sec.wtr_acc_min = 0.;

            (duration_minutes, new_deficit) = calc_dur_and_deficit(wtr_cfg, run_sec, db, sec, sec_start);
            // Wizard mode don't water if less than for 1 minute, that would not have any practical effect, saving wtr engine, water, energy, maintenance costs, ...
            // Any calculated value is already in the new_deficit value calculated above.
            if duration_minutes >= 1. {
                sec_start = upd_run_sec_data(run_sec, sec_start, cycle.run.cycle_id,cycle.run.run_id, duration_minutes, pump_recycle_secs);
                wtr_secs.push_back((run_sec.sec_id, run_sec_ptr as u8));
            }
            sec.deficit = new_deficit;
        }

        // update watering cycle - log error but continue.
        db.upd_cycle_srvr(cycle).unwrap_or_else(|e| self.errors.push(Box::new(e)));
        db.ins_cycle_run(&cycle.run).unwrap_or_else(|e| self.errors.push(Box::new(e)));
        for (_, sec_ptr) in self.wtr_secs.iter() {
            db.ins_secs_run(&self.run_secs[*sec_ptr as usize]).unwrap_or_else(|e| self.errors.push(Box::new(e)));
        }
        // update active_ptr if we have more sectors to water
        if let Some((sec_id, sec_ptr)) = self.wtr_secs.front() {
            self.active_ptrs.sec_id = Some(*sec_id);
            self.active_ptrs.run_sec_ptr = Some(*sec_ptr);
        } else {
            self.active_ptrs.reset_sec();
        }
    }

    #[inline]
    fn start_sec(&mut self, is_manual: bool) {
        let sector_id: SECTOR_ID;
        let run_sec_ptr: SECTOR_PTR;

        if !is_manual {
            (sector_id, run_sec_ptr) = self.wtr_secs.pop_front().unwrap();
        } else {
            sector_id = self.active_ptrs.sec_id.unwrap();
            run_sec_ptr = self.active_ptrs.run_sec_ptr.unwrap();
        }

        let mut run_sector = &mut self.run_secs[run_sec_ptr as usize];
        let sec = &mut self.sectors[sector_id as usize];
        let valve_result = turn_on_sec(self.dev_svc.clone(), sec.device_id,  sec.name.to_owned());
        
        upd_in_mem_run_sec(run_sector, valve_result.1, WateringStatus::Running, &mut self.errors);
        run_sector.is_manual = is_manual;

        // adjust the end time as with the wtr_acc_min because:
        //  if it is zero. means a fresh start or watering end-to-end with no stops
        //  otherwise, means that there was suspend time
        // REVIEW porque estas contas parecem estar erradas para a lógica pretendida
        run_sector.end = run_sector.start.add_secs_f32(min_to_sec_f32(run_sector.wtr_tgt_min - run_sector.wtr_acc_min));

        run_sector.last_start = run_sector.start;
        sec.state = valve_result.0;
        disable_if_not_ok(sec);
        sec.last_change = run_sector.start;
        upd_in_db_sec(&self.db, run_sector, sec, &mut self.errors);
        self.active_ptrs.sec_id = Some(sector_id);
        self.active_ptrs.run_sec_ptr = Some(run_sec_ptr);

    }
}

