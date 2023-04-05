use crate::data_structs::rega::{command::*, running_ptr::*, state::*};
use crate::services::irrigation::{actions::start::*, wtr_engine::*};
use crate::{app_time::ctrl_time::*, log_warn, logger::*};
use ctrl_prelude::string_resources::*;

pub trait WizardWait {
    fn wizard_wait(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl WizardWait for WtrEngine {
    /// Wizard mode waiting for any cycle timmer timeout
    #[inline]
    fn wizard_wait(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::WzrWait;
        let (nxt, interrupt, watered_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match *cmd {
                Command::StartCycle(cycle_ptr) => {
                    _nxt = State::WzrWtrCycle;
                    self.wtr_cfg.changed = true;
                    self.start_cycle_not_manual(cycle_ptr);

                    let cycle = &mut self.cycles[cycle_ptr as usize];
                    match self.active_ptrs {
                        RunningPtr {
                            cycle: Some(_),
                            sec_id: Some(_),
                            ..
                        } => {
                            // Se existirem skipped sectors quando for correr o wizard
                            // vai reavaliar as necessidades de agua de todos os setores e reconstruir os run sectors
                            // no momento do arranque do ciclo
                            // // else
                            // if no skipped sectors, nothing to do
                            // Also by reqs definiton we should water 1 time/day, unless  emergency watering is needed
                            //(in which case means 2 times/day max)
                            // Other situations should be signaled/supervised, and one must act accordingly
                            self.snd_command(Command::StartSector);
                        }
                        RunningPtr {
                            cycle: Some(_),
                            sec_id: None,
                            ..
                        } => {
                            self.cmd_queue.push_back(Command::EndCycle(cycle_ptr));
                            // Só é erro no modo standard e manual - no modo wizard é porque se saltaram todos os setores.
                            log_warn!(err_cycle_start(&cycle.name));
                        }
                        _ => unreachable!(), //qq coisa errada no codigo para chegar aqui
                    }
                }
                Command::ChangeState => {
                    _nxt = State::EstablishMode;
                }
                _ => (), //other commands are ignored - no transition
            }
        } else {
            self.wtr_cfg.changed = true;
            self.process_interrupt(time, watered_status);
        }
        self.validate_errors(_nxt)
    }
}
