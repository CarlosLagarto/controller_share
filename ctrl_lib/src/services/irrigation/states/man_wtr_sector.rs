use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::{actions::stop::*, wtr_engine::*};

pub trait ManualWateringSector {
    fn manual_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl ManualWateringSector for WtrEngine {
    /// Manually watering sector
    #[inline]
    fn manual_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let (mut nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        if !interrupt {
            match *cmd {
                Command::EndSector(_) | Command::StopSector(_) => {
                    nxt = State::ManWtrCycle;
                    self.wtr_cfg.changed = true;
                    self.stop_sec(time);
                    if self.wtr_secs.is_empty() {
                        nxt = State::ManWait;
                    }
                }
                Command::StopCycle(cycle_ptr) => {
                    nxt = State::ManWait;
                    self.wtr_cfg.changed = true;
                    self.stop_man_watering_cycle(Some(cycle_ptr), time, WateringStatus::Terminated);
                }
                _ => (), //other states are ignored - no transition
            }
        } else {
            self.wtr_cfg.changed = true;
            self.stop_man_watering_cycle(None, time, watering_status);
        }
        self.validate_errors(nxt)
    }
}
