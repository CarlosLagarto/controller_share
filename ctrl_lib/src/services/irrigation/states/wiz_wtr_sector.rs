use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::actions::{stop::*, suspend::*};
use crate::services::irrigation::wtr_engine::*;

pub trait WizardWateringSector {
    fn wizard_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl WizardWateringSector for WtrEngine {
    /// Está a executar o ciclo de rega - watering the sector - in wizard mode
    #[inline]
    fn wizard_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::WzrWtrSector;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match cmd {
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
                Command::EndSector(_) => {
                    _nxt = State::WzrWtrCycle;
                    self.wtr_cfg.changed = true;
                    if self.wtr_secs.is_empty() {
                        self.snd_command(Command::EndCycle(self.active_ptrs.cycle.unwrap()));
                    } else {
                        self.stop_sec(time);
                    }
                }
                _ => (), //ignoramos o resto
            }
        } else {
            self.wtr_cfg.changed = true;
            self.process_interrupt(time, watering_status);
        }
        self.validate_errors(_nxt)
    }
}
