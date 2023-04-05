use crate::app_time::ctrl_time::*;
use crate::data_structs::msgs::int_message::*;
use crate::data_structs::rega::{command::*, mode::*, state::*};
use crate::services::irrigation::{wtr_engine::*, db_model::*};

pub trait EstablishMode {
    fn establish_mode(&mut self, cmd: &Command, time: CtrlTime) -> State;
}

impl EstablishMode for WtrEngine {
    /// iterin state - helper to configure machine and determine next state, depending on configurations
    #[inline]
    fn establish_mode(&mut self, cmd: &Command, time: CtrlTime) -> State {
        let mut nxt = State::EstablishMode;
        let wtr_cfg = &mut self.wtr_cfg;
        // neste estado o state não têm que ser um fator para interrupção, porque estamos no próprio estado e dessa forma nunca entravamos na logica
        let (state, interrupt) = match *cmd {
            Command::ShutDown => (State::Shutdown, true),
            Command::Error => (State::Error, true),
            _ => (wtr_cfg.state, false),
        };

        if !interrupt || state != nxt {
            let mut _mode = Mode::Manual;
            if let Command::ChangeMode(mode) = *cmd {
                _mode = mode;
            } else if *cmd == Command::Start {
                _mode = wtr_cfg.mode;
            } else {
                return State::ManWait;
            }
            wtr_cfg.mode = _mode;
            // Estabelece o modo da maquina
            nxt = match _mode {
                Mode::Manual => State::ManWait,
                Mode::Wizard => State::WzrWait,
                Mode::Standard => {
                    if !self.std_ptrs.is_empty() {
                        State::StdWait // só vai para o standard se houver ciclos standard definidos
                    } else {
                        State::NoScheduleDef
                    }
                }
            };
            // Percorre todos os ciclos, mas é só chamada no estado establish mode, e aí a idéia é mesmo reatualizar tudo.
            // vamos procurar os schedules ativos ...
            // Quando se termina o cycle/ciclo, tem que se atualizar a data de start para a proxima data
            let mut _processed = 0;
            for cycle in self.cycles.iter_mut() {
                if time > cycle.schedule.start {
                    // só atualizamos se o tempo já passou.  Caso contrário saltava-se um ciclo...pelo menos pelos testes no modo wizard
                    match cycle.get_next_event(_mode, time, &self.wtr_cfg.geo_pos) {
                        Ok(Some(next_start)) => {
                            cycle.schedule.start = next_start;
                            //persist in db and do other stuff
                            self.db.upd_cycle_srvr(cycle).unwrap_or_else(|e| self.errors.push(Box::new(e)));
                        }
                        Ok(None) => {} //nop
                        Err(err) => self.errors.push(Box::new(err)),
                    };
                }
            }

            nxt = self.validate_errors(nxt);
            self.msg_broker.reg_int_msg(MsgData::WaterMachineStarted, time);
        } else {
            nxt = state;
        }
        self.wtr_cfg.changed = true;
        nxt
    }
}
