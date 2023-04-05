use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::actions::{start::*, stop::*};
use crate::services::irrigation::wtr_engine::*;
use crate::app_time::ctrl_time::*;

pub trait StandardWateringCycle {
    fn standard_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl StandardWateringCycle for WtrEngine {
    /// Standard mode cycle executing
    #[inline]
    fn standard_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::StdWtrCycle;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match *cmd {
                Command::StartSector => {
                    _nxt = State::StdWtrSector;
                    self.wtr_cfg.changed = true;
                    self.start_sec(false);
                }
                Command::StopCycle(_) => {
                    _nxt = State::StdWait;
                    self.wtr_cfg.changed = true;
                    self.process_interrupt(time, WateringStatus::Terminated);
                }
                Command::EndCycle(_) => {
                    _nxt = State::StdWait;
                    self.wtr_cfg.changed = true;
                    self.stop_cycle(time, WateringStatus::Terminated);
                }
                // Command::StartCycle(_) => {
                //     log_warn!("Existem ciclos standard com sobreposição no arranque.");
                // }
                _ => (), //other states are ignored - no transition
            }
        } else {
            self.wtr_cfg.changed = true;
            self.process_interrupt(time, watering_status);
        }
        self.validate_errors(_nxt)
    }
}
