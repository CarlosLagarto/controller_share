use crate::data_structs::rega::{command::*, state::*};
use crate::services::irrigation::{actions::start::*, wtr_engine::*};
use crate::{app_time::ctrl_time::*, log_warn, logger::*};

use ctrl_prelude::string_resources::*;

pub trait StandardWait {
    fn standard_wait(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl StandardWait for WtrEngine {
    /// Standard mode waiting for any cycle timer timeout
    #[inline]
    fn standard_wait(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::StdWait;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match *cmd {
                Command::StartCycle(cycle_ptr) => {
                    _nxt = State::StdWtrCycle;
                    self.wtr_cfg.changed = true;
                    self.start_cycle_not_manual(cycle_ptr);
                    if self.active_ptrs.sec_id.is_some() {
                        self.cmd_queue.push_back(Command::StartSector);
                    } else {
                        self.cmd_queue.push_back(Command::EndCycle(cycle_ptr));
                        log_warn!(err_cycle_start(&self.cycles[cycle_ptr as usize].name))
                    }
                }
                Command::ChangeState => {
                    _nxt = State::EstablishMode;
                }
                _ => (), //other states are ignored - no transition
            }
        } else {
            self.wtr_cfg.changed = true;
            self.process_interrupt(time, watering_status);
        }
        self.validate_errors(_nxt)
    }
}
