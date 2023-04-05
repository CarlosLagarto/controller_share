use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, running_ptr::*, state::*, watering_status::*};
use crate::services::irrigation::wtr_engine::*;
use crate::{log_error, logger::*};

pub trait Error {
    fn error(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
//
impl Error for WtrEngine {
    /// Everything unknown or unrecoverable will transition the machine to this state
    #[inline]
    fn error(&mut self, cmd: &Command, time: CtrlTime) -> State {
        match self.active_ptrs {
            RunningPtr {
                cycle: Some(_),
                sec_id: Some(sec_id),
                run_sec_ptr: Some(run_sec_ptr),
            } => {
                let run_sector = self.run_secs.get_mut(run_sec_ptr as usize).unwrap();
                let sec = &mut self.sectors[sec_id as usize];
                if run_sector.is_watering() {
                    let valve_result = turn_off_sec(self.dev_svc.clone(), sec.device_id, sec.name.clone());
                    run_sector.status = WateringStatus::Error;
                    if let Some(e) = valve_result.1{
                        push_error( run_sector, &mut self.errors, e );
                    }
                    upd_in_mem_sec_on_stop(run_sector, sec, valve_result.0, time);
                }
            }
            RunningPtr {
                cycle: Some(_),
                sec_id: None,
                ..
            } => {
                //TODO - pode não haver um setor ativo, mas ainda assim estar um ciclo ativo e deve haver dados a atualizar.....
            }
            _ => {} //nop
        }
        self.active_ptrs.reset_all();

        let mut msg : String;
        while !self.errors.is_empty() {
            let err = &*self.errors.pop().unwrap();
            msg = err.to_string();
            log_error!(&msg);
            self.msg_broker.snd_error_to_client(&msg);
        }

        match *cmd {
            Command::ChangeMode(_) => State::EstablishMode,
            Command::ShutDown => State::Shutdown,
            _ => State::Error,
        }
    }
}
