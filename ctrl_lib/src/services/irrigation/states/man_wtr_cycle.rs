use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::actions::{start::*, stop::*};
use crate::services::irrigation::wtr_engine::*;

pub trait ManualWateringCycle {
    fn manual_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl ManualWateringCycle for WtrEngine {
    /// Manually executing cycle
    #[inline]
    fn manual_watering_cycle(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let (mut nxt, interrupt, watering_state) = self.is_interrupt_and_change_command(cmd);
        if !interrupt {
            match *cmd {
                Command::StartSector => {
                    self.wtr_cfg.changed = true;
                    nxt = State::ManWtrSector;
                    self.start_sec(false);
                }
                Command::StopCycle(cycle_id) => {
                    self.wtr_cfg.changed = true;
                    nxt = State::ManWait;
                    self.stop_man_watering_cycle(Some(cycle_id), time, WateringStatus::Terminated);
                }
                Command::EndCycle(_) => {
                    nxt = State::ManWait;
                    self.wtr_cfg.changed = true;
                    self.stop_cycle(time, WateringStatus::Terminated);
                }
                _ => (), //other states are ignored - no transition
            }
        } else {
            self.wtr_cfg.changed = true;
            self.stop_man_watering_cycle(None, time, watering_state);
        }
        self.validate_errors(nxt)
    }
}
