use crate::services::irrigation::wtr_engine::*;
use crate::data_structs::rega::{state::*, command::*};

pub trait NoScheduleDef {
    fn no_schedule_def(&mut self, cmd: &Command) -> State;
}
impl NoScheduleDef for WtrEngine {
    /// ditto
    #[inline]
    fn no_schedule_def(&mut self, cmd: &Command) -> State {
        // mantêm-se neste estado até estar definido o cycle
        let mut _nxt = State::NoScheduleDef;
        let (_state, interrupt, _) = self.is_interrupt_and_change_command(cmd);

        match *cmd {
            Command::ChangeState if !interrupt => _nxt = State::EstablishMode,
            _ => _nxt = _state,
        };
        _nxt
    }
}
