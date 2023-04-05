use crate::app_time::ctrl_time::CtrlTime;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::{actions::stop::*, wtr_engine::*};

pub trait ManualWateringSectorDirect {
    fn manual_watering_sector_direct(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl ManualWateringSectorDirect for WtrEngine {
    /// Está a executar o ciclo de rega
    #[inline]
    fn manual_watering_sector_direct(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::ManWtrSectorDirect;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            #[allow(clippy::single_match)] //for future growth
            match *cmd {
                Command::StopSector(_) => {
                    _nxt = State::ManWait;
                    self.wtr_cfg.changed = true;
                    self.stop_cycle(time, WateringStatus::Terminated);
                    self.active_ptrs.reset_all();
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
