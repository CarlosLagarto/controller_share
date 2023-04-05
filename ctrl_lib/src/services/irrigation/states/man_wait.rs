use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::*, state::*};
use crate::services::irrigation::{actions::start::*, wtr_engine::*};

pub trait ManualWait {
    fn manual_wait(&mut self, cmd: &Command, time: CtrlTime) -> State;
}
impl ManualWait for WtrEngine {
    /// Manual mode - waiting for manual state change
    #[inline]
    fn manual_wait(&mut self, cmd: &Command, time: CtrlTime) -> State {
        // Aguarda que se altere o estado
        let mut nxt = State::ManWait;
        // para se tirar do erro tem que se por a máquina em modo manual
        let (state, interrupt, _) = self.is_interrupt_and_change_command(cmd);
        if !interrupt {
            match cmd {
                // Só os ciclos diretos é que podem ser forçados, e by design só existe 1 ciclo directo, pelo que não é preciso o id
                Command::ForceCycle(_) => {
                    nxt = State::ManWtrCycle;
                    self.wtr_cfg.changed = true;
                    // isto é comando apenas possivel sobre os ciclos direct
                    let cycle_ptr = self.internal.direct.unwrap();//.get_std_cycle_by_id(*cycle_id).ptr.unwrap();
                    self.active_ptrs.cycle = Some(cycle_ptr);
                    // self.start_cycle(false, None);
                    self.start_cycle(true, Some(time));
                    if self.active_ptrs.sec_id.is_some() {
                        self.snd_command(Command::StartSector);
                    } else {
                        self.snd_command(Command::EndCycle(cycle_ptr));
                    }
                }
                Command::ForceSector(sector_id) => {
                    if self.sectors[*sector_id as usize].enabled {
                        self.wtr_cfg.changed = true;
                        nxt = State::ManWtrSectorDirect;
                        // adicionamos apenas este setor aos run sectors do cycle interno direct e ajustar os dados de arranque
                        // com a ultima alteração asseguramos que o ciclo interno directo já existe, pelo que é só adicionar o setor pretendido aos running secs
                        let cycle_ptr = self.internal.direct.unwrap();
                        self.active_ptrs.cycle = Some(cycle_ptr);
                        self.active_ptrs.sec_id = Some(*sector_id);
                        self.start_cycle(true, Some(time));

                        self.start_sec(true);
                    } else {
                        // nop - não faz nada porque não vamos ligar um setor disabled.  Se puder ser ativado, deve-se colocar em enabled primeiro
                    }
                }
                Command::ChangeState => {
                    nxt = State::EstablishMode;
                }
                _ => (), //other states are ignored - no transition
            }
        } else if interrupt {
            nxt = state;
            self.wtr_cfg.changed = true;
            
        }
        self.validate_errors(nxt)
    }
}
