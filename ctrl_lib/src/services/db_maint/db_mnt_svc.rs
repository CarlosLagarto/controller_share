use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::config::app_config::*;
use crate::data_structs::msgs::int_message::*;
use crate::db::db_error::*;
use crate::services::{db_maint::db_model::*, msg_broker::msg_brkr_svc::*};
use crate::{controller_sync::*, log_error, log_info, log_warn, logger::*};

use ctrl_prelude::{error::*, globals::GIGA_U, string_resources::*};

// # nanos for the delay - 10 seconds
pub const DB_MAINT_START_DELAY: u64 = 10 * GIGA_U;

/// Dimension 72
pub struct DBMntSvc {
    pub large_mnt_schedule: Schedule,
    pub daily_mnt_schedule: Schedule,
}

impl DBMntSvc {
    #[inline]
    pub fn new(time: CtrlTime, app_cfg: &mut AppCfg) -> Self {
        // init the struct with the signal set so all the other threads can work
        // mnt routines start 10 secs after the end of the day
        // other services must check the signal - for my use case only the database working processes need to coordinate,
        // weather process starts 5 secs earlier
        DB_MAINT_SIG.read().set();
        let start_daily_mnt: CtrlTime = adjust_time_for_delay(time.eod_ux_e());
        let start_large_mnt: CtrlTime = if DBMntSvc::is_running_for_first_time(app_cfg) {
            // first time running, find end of day and add db_maint_days
            adjust_time_for_delay(time.eod_ux_e().add_days((app_cfg.db_maint.db_mnt_days - 1) as u64))
        } else {
            // else, check how many days elapsed since las stop and subtract from the defined max
            let mut nr_of_days_to_add: i8 = app_cfg.db_maint.db_mnt_days as i8 - app_cfg.db_maint.db_mnt_counter as i8;
            if nr_of_days_to_add < 0 {
                // this may happen during tests and simulation modes, where frequent start/stop without finishing the logic
                // may increment the counter above the defined value.  In prd should not happen...in thesis :-)
                app_cfg.db_maint.db_mnt_counter = 0;
                nr_of_days_to_add = 0;
            }
            time.eod_ux_e().add_days(nr_of_days_to_add as u64);
            app_cfg.db_maint.db_mnt_last_run.add_days(app_cfg.db_maint.db_mnt_days as u64)
        };

        let large_mnt_schedule = Schedule::build_run_forever(start_large_mnt, app_cfg.db_maint.db_mnt_days as u16, ScheduleRepeatUnit::Days);
        let daily_mnt_schedule = Schedule::build_run_forever(start_daily_mnt, 1u16, ScheduleRepeatUnit::Days);
        Self {
            large_mnt_schedule,
            daily_mnt_schedule,
        }
    }

    /// Called every day accordingly with the defined schedule
    ///
    /// Copy to the backup db the info with more than "db_maint_days" days old.
    /// Clean logs and analyze db stats for queries performance
    ///
    /// Objetive is to keep "operational" DB, lean and clean for performance issues
    ///
    #[inline]
    pub fn verify_things_to_do(&mut self, time: CtrlTime, msg_broker: &MsgBrkr, app_cfg: &mut AppCfg) {
        if !self.large_mnt_schedule.is_time_to_run(time) {
            // daily schedule only runs when is not time to run the large maintenace
            if self.daily_mnt_schedule.is_time_to_run(time) {
                self.do_daily_mnt(app_cfg, time, msg_broker);
            }
        } else {
            self.do_large_mnt(app_cfg, time, msg_broker);
        }
    }

    #[inline]
    fn do_daily_mnt(&mut self, app_cfg: &mut AppCfg, time: CtrlTime, msg_broker: &MsgBrkr) {
        start_mnt(app_cfg, msg_broker, time, &mut self.daily_mnt_schedule, daily_maintenance);
        end_mnt(msg_broker, time);
    }

    #[inline]
    fn do_large_mnt(&mut self, app_cfg: &mut AppCfg, time: CtrlTime, msg_broker: &MsgBrkr) {
        start_mnt(app_cfg, msg_broker, time, &mut self.large_mnt_schedule, backup_and_maintenance);
        // DESIGN NOTE 
        // we may have a situation where the machine is stopped for some time, and when evaluating the 20 days
        // (or whatever is configured) still be under the actual time and it may make no sense to run the procedure again just shortly after
        Self::adjust_schedule_if_program_stopped_for_long_time(&mut self.large_mnt_schedule, time, "periódico");

        app_cfg.reset_db_maint_counter();
        // advance also the daily schedule, because when running the large procedure, we skip the daily one.
        Self::adjust_schedule_if_program_stopped_for_long_time(&mut self.daily_mnt_schedule, time, "diário");

        end_mnt(msg_broker, time);
    }

    #[inline]
    fn adjust_schedule_if_program_stopped_for_long_time(schedule: &mut Schedule, time: CtrlTime, schedule_name: &str) {
        if schedule.start < time {
            let next_event = find_next_event(time, schedule);
            match next_event {
                Ok(Some(next_time)) => schedule.start = next_time,
                Ok(None) => warn!(
                    "Tema no cálculo da data da próxima manutenção do schedule {}.  Tem que se perceber porquê e ajustar o que for necessário.",
                    schedule_name
                ),
                Err(e) => {
                    warn!(
                        "Tema no cálculo da data da próxima manutenção do schedule {}.  Tem que se perceber porquê e ajustar o que for necessário.",
                        schedule_name
                    );
                    log_warn!(e);
                }
            }
        }
    }

    /// return true if db_maint_last_run == 0
    #[inline]
    pub fn is_running_for_first_time(app_cfg: &AppCfg) -> bool {
        app_cfg.db_maint.db_mnt_last_run == CtrlTime(0)
    }
}

#[inline]
#[rustfmt::skip]
pub fn adjust_time_for_delay(time: CtrlTime) -> CtrlTime { time + DB_MAINT_START_DELAY }

#[rustfmt::skip]
#[inline]
fn start_mnt(app_cfg: &mut AppCfg, msg_broker: &MsgBrkr, time: CtrlTime, schedule: &mut Schedule, mnt_fn: fn(CtrlTime) -> SimpleResult) {
    wait_for_db_maint_task();  // in the remote hypothesis (some big/strange problem happend) that previous task is still running
    // signal de world that we are processing so during db maintenance, everyone is on hold
    DB_MAINT_SIG.read().reset();
    app_cfg.set_increment_db_maint_counter();
    let char8_date: &str = &time.as_date_char8_str_e();
    msg_broker.reg_int_msg(MsgData::DBMaintStarted, time);
    match mnt_fn(time) {
        Ok(_) => app_cfg.set_db_maint_last_run(time),
        Err(err) => {
            let msg = err_db_mnt_script(char8_date);
            log_info!(&msg);
            log_error!(build_error(&err));
            msg_broker.snd_error_to_client(&msg);
        }
    }
    // set next event time
    _ = schedule.set_next_event().map_err(|e| log_error!(build_error(&e)));
}

#[inline]
fn end_mnt(msg_broker: &MsgBrkr, time: CtrlTime) {
    // signal de world that the process ended
    DB_MAINT_SIG.read().set();
    msg_broker.reg_int_msg(MsgData::DBMaintCompleted, time);
}
