use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::config::app_config::*;
use crate::controller_sync::DB_MAINT_SIG;
use crate::data_structs::msgs::int_message::*;
use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::{db_maint::db_model::*, msg_broker::svc::*};
use crate::{log_error, log_info, logger::*};

use ctrl_prelude::error::build_error;
use ctrl_prelude::{globals::GIGA_U, string_resources::*};

// nr de nanos para o delay - 10 segundos neste caso
pub const DB_MAINT_START_DELAY: u64 = 10 * GIGA_U;

/// Dimensão 40
pub struct DBMntSvc {
    pub large_mnt_schedule: Schedule,
    pub daily_mnt_schedule: Schedule,
    db: Persist,
}

impl DBMntSvc {
    #[inline]
    pub fn new(time: CtrlTime, db: Persist, app_cfg: &mut AppCfg) -> Self {
        // começa com o signal aberto para o resto do mundo poder avançar
        // começa a manutenção 10 segundos depois do fim do dia.
        // articular com outros serviços - neste caso o unico diário é a metereologia, que será 5 segundos antes
        DB_MAINT_SIG.read().set();
        let start_daily_mnt: CtrlTime = adjust_time_for_delay(time.eod_ux_e());
        let start_large_mnt: CtrlTime = if DBMntSvc::is_running_for_first_time(app_cfg) {
            // como é a primeira vez, encontramos o fim do dia, e somamos os db_maint_days definidos
            adjust_time_for_delay(time.eod_ux_e().add_days((app_cfg.db_maint.db_mnt_days - 1) as u64))
        } else {
            // senão, temos que ir ver quantos dias correu antes da ultima paragem, e subtrai-los ao max definido
            time.eod_ux_e().add_days(app_cfg.db_maint.db_mnt_days as u64 - app_cfg.db_maint.db_mnt_counter as u64);
            app_cfg.db_maint.db_mnt_last_run.add_days(app_cfg.db_maint.db_mnt_days as u64)
        };

        let large_mnt_schedule = Schedule::build_run_forever(start_large_mnt, app_cfg.db_maint.db_mnt_days as u16, ScheduleRepeatUnit::Days);
        let daily_mnt_schedule = Schedule::build_run_forever(start_daily_mnt, 1u16, ScheduleRepeatUnit::Days);
        Self { large_mnt_schedule, daily_mnt_schedule, db }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn is_time_to_run_large_mnt(&self, time: CtrlTime) -> bool { time >= self.large_mnt_schedule.start }

    #[inline]
    #[rustfmt::skip]
    pub fn is_time_to_run_daily_mnt(&self, time: CtrlTime) -> bool { time >= self.daily_mnt_schedule.start }
    /// É chamado todos os dias periodicamente de acordo com os schedules
    ///
    /// Copy to the backup db the info with more than "db_maint_days" days old.
    /// Clean logs and analyze db stats for queries performance
    ///
    /// Objetive is to keep "operational" DB, lean and clean for performance issues
    ///
    #[inline]
    pub fn process_time_tick(&mut self, time: CtrlTime, msg_broker: &MsgBrkr, app_cfg: &mut AppCfg) {
        if !self.is_time_to_run_large_mnt(time) {
            // só corre este se não fôr dia de large maintenace
            if self.is_time_to_run_daily_mnt(time) {
                self.do_daily_mnt(app_cfg, time, msg_broker);
            }
        } else {
            self.do_large_mnt(app_cfg, time, msg_broker);
        }
    }

    #[inline]
    fn do_daily_mnt(&mut self, app_cfg: &mut AppCfg, time: CtrlTime, msg_broker: &MsgBrkr) {
        Self::start_mnt(app_cfg, msg_broker, &self.db, time, &mut self.daily_mnt_schedule, DESC_DB_MNT_START_DAILY_MNT, DBModelTech::daily_maintenance);
        self.end_mnt(msg_broker, time, DESC_DB_MNT_END_DAILY_MNT);
    }

    #[inline]
    fn do_large_mnt(&mut self, app_cfg: &mut AppCfg, time: CtrlTime, msg_broker: &MsgBrkr) {
        Self::start_mnt(app_cfg, msg_broker, &self.db, time, &mut self.large_mnt_schedule, DESC_DB_MNT_START_LARGE_MNT, DBModelTech::backup_and_maintenance);
        app_cfg.reset_db_maint_counter();
        // e avançamos também o daily schedule, uma vez que no dia em que se executa o large, saltou-se este
        _ = self.daily_mnt_schedule.set_next_event().map_err(|e| log_error!(e.to_string()));
        self.end_mnt(msg_broker, time, DESC_DB_MNT_END_LARGE_MNT);
    }

    /// devolve true se o parametro db_maint_last_run == 0
    #[inline]
    pub fn is_running_for_first_time(app_cfg: &AppCfg) -> bool {
        app_cfg.db_maint.db_mnt_last_run == CtrlTime(0)
    }

    #[rustfmt::skip]
    #[inline]
    fn start_mnt(app_cfg: &mut AppCfg, msg_broker: &MsgBrkr, db: &Persist, time: CtrlTime, schedule: &mut Schedule, description: &str, mnt_fn: fn(&Persist, CtrlTime) -> SimpleResult) {
        // signal de world that we are processing so during db maintenance, everyone is on hold
        DB_MAINT_SIG.read().reset();
        app_cfg.set_increment_db_maint_counter();
        let char8_date: &str = &time.as_date_char8_str_e();
        msg_broker.reg_int_msg(MsgData::DBMaintStarted, time, description);
        match mnt_fn(db, time) {
            Ok(_) => app_cfg.set_db_maint_last_run(time),
            Err(err) => {
                let msg = err_db_mnt_script(char8_date);
                log_info!(&msg);
                log_error!(build_error(&err));
                msg_broker.snd_error_to_clients(&msg, "");
            }
        }
        //avançamos para o evento seguinte.
        _ = schedule.set_next_event().map_err(|e| log_error!(build_error(&e)));
    }

    #[inline]
    fn end_mnt(&mut self, msg_broker: &MsgBrkr, time: CtrlTime, description: &str) {
        // signal de world that the process ended
        DB_MAINT_SIG.read().set();
        msg_broker.reg_int_msg(MsgData::DBMaintCompleted, time, description);
    }
}

#[inline]
#[rustfmt::skip]
pub fn adjust_time_for_delay(time: CtrlTime) -> CtrlTime { time + DB_MAINT_START_DELAY }
