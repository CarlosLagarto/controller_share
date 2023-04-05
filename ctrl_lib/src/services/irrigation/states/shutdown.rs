use crate::data_structs::rega::state::*;
use crate::services::irrigation::wtr_engine::*;

pub trait Shutdown {
    fn shut_down(&self) -> State;
}
impl Shutdown for WtrEngine {
    /// Executa o shutdown controlado da máquina
    #[inline]
    #[rustfmt::skip]
    fn shut_down(&self) -> State { State::Shutdown }
}
