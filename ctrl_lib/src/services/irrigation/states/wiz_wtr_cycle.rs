use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::actions::{start::*, stop::*, suspend::*};
use crate::services::irrigation::wtr_engine::*;

pub trait WizardWateringCycle {
    fn wizard_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl WizardWateringCycle for WtrEngine {
    /// Executing cycle schedule in the wizard mode
    #[inline]
    fn wizard_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::WzrWtrCycle;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match cmd {
                Command::StartSector => {
                    _nxt = State::WzrWtrSector;
                    self.wtr_cfg.changed = true;
                    self.start_sec(false);
                }
                Command::EndCycle(_) => {
                    _nxt = State::WzrWait;
                    self.wtr_cfg.changed = true;
                    self.stop_cycle(time, WateringStatus::Terminated);
                }
                Command::Suspend(_alert) => {
                    _nxt = State::SuspendedWizard;
                    self.wtr_cfg.changed = true;
                    self.suspend(time);
                }
                Command::StopCycle(_) => {
                    _nxt = State::WzrWait;
                    self.wtr_cfg.changed = true;
                    self.process_interrupt(time, WateringStatus::Terminated);
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
