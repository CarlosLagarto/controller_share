use crate::data_structs::rega::{command::*, state::*, watering_status::*};
use crate::services::irrigation::{actions::stop::*, wtr_engine::*};
use crate::{app_time::ctrl_time::*, log_warn, logger::*};

pub trait StandardWateringSector {
    fn standard_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl StandardWateringSector for WtrEngine {
    /// Standard mode - cycle - sector executing
    #[inline]
    fn standard_watering_sector(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut _nxt = State::StdWtrSector;
        let (nxt, interrupt, watering_status) = self.is_interrupt_and_change_command(cmd);
        _nxt = nxt;
        if !interrupt {
            match *cmd {
                Command::EndSector(_) => {
                    _nxt = State::StdWtrCycle;
                    self.wtr_cfg.changed = true;
                    if self.wtr_secs.is_empty() {
                        self.snd_command(Command::EndCycle(self.active_ptrs.cycle.unwrap()));
                    } else {
                        self.stop_sec(time);
                    }
                }
                Command::StopCycle(_) => {
                    _nxt = State::StdWait;
                    self.wtr_cfg.changed = true;
                    self.process_interrupt(time, WateringStatus::Terminated);
                }
                Command::StartCycle(_) => {
                    log_warn!("Existem ciclos standard com sobreposição no arranque.");
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
