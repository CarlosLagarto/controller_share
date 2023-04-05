use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::actions::{stop::*, suspend::*};
use crate::services::irrigation::wtr_engine::*;

pub trait SuspendedWizard {
    fn suspended_wizard(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl SuspendedWizard for WtrEngine {
    /// Aguarda pelo trigger do evento de rega, com base no cycle definido
    #[inline]
    fn suspended_wizard(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::SuspendedWizard;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match *cmd {
                Command::Resume => {
                    self.suspend_timeout = None;
                    self.wtr_cfg.changed = true;
                    if self.resume(time) {
                        _nxt = State::WzrWtrSector;
                    } else {
                        _nxt = State::WzrWtrCycle;
                        self.snd_command(Command::EndCycle(self.active_ptrs.cycle.unwrap()));
                    }
                }
                Command::ResumeTimeOut => {
                    self.suspend_timeout = None;
                    self.wtr_cfg.changed = true;
                    _nxt = State::WzrWait;
                    self.stop_cycle(time, WateringStatus::SuspendedTimeout);
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
