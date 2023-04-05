/// A Máquina de estados têm a seguinte definição
///                             Comandos  ->  Estado Futuro                                 (Man|Std|Wrd)  (Man|Std|Wrd)  (Man|Std|Wrd)
/// Estado atual                            Starting        EstablishMode   NoScheduleDef       Wait          Cycle            Sec       SecDirect  Suspend   ShutDown   Error
///
/// Starting                Start           X (no arranque)      X           ->  X                                                    
///  .............................................................................. Consoante o modo da máquina e as condições dos ciclos
/// EstablishMode         -- transição para Prep. ciclos e secs              ->  X               X
///
/// No Schedule Def       ChangeMode(MODE)                        X        
///                       ChangeState                             X
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
/// Manual Wait           ChangeMode(MODE)                        X        
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       ForceCycle                                                                           X   
///                       ForceSector                                                                                                      X                                                      
///                       ChangeState                             X
/// Man Wtr Cycle         ChangeMode(MODE)                        X
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StartSector                                                                                           X
///                       StopCycle                                                               X
///                       EndCycle                                                                X
/// Man Wtr Sector        ChangeMode(MODE)                        X
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       EndSector                                                                            X
///                       StopSector                                                                           X
///                       StopCycle                                                               X
/// Man Wtr Sector Dir    ChangeMode(MODE)                        X               
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StopSector                                                              X
/// Standard Wait         ChangeMode(MODE)                        X                        
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StartCycle                                                                          X
///                       ChangeState                             X
/// Std Wtr Cycle         ChangeMode(MODE)                        X                       
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StartSector                                                                                          X
///                       StopCycle                                                               X
///                       EndCycle                                                                X
/// Std Wtr Sector        ChangeMode(MODE)                        X                    
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       EndSector                                                                           X
///                       StopCycle                                                               X
/// Wizard Wait           ChangeMode(MODE)                        X                  
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StartCycle                                                                          X
///                       ChangeState                             X
/// Wzrd Wtr Cycle        ChangeMode(MODE)                        X                  
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       StartSector                                                                                         X
///                       StopCycle                                                                                  X
///                       EndCycle                                                                                   X
///                       Suspend                                                                                                                      X
/// Wzrd Wtr Sector       ChangeMode(MODE)                        X                   
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       EndSector                                                                          X
///                       StopSector                                                                         X
///                       StopCycle                                                                                  X
///                       Suspend                                                                                                                      X
/// Wzrd Suspended        ChangeMode(MODE)                        X                      
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
///                       Resume                                                                             X
///                       ResumeTimeout                                                                              X
///
/// Error                 ChangeMode(MODE)                        X                        
///                       ShutDown                                                                                                                              X
///                       Error                                                                                                                                             X
/// Shutdown            -------------      Qualquer estado pode saltar para o estado de shutdown - estado terminal  ----------------
///
use crate::app_time::ctrl_time::*;
use crate::data_structs::rega::{command::Command, running_ptr::*, state::*};
use crate::services::irrigation::states::{error::*, establish_mode::*, man_wait::*, man_wtr_cycle::*, man_wtr_sector::*, man_wtr_sector_direct::*};
use crate::services::irrigation::states::{no_schedule_def::*, shutdown::*, starting::*, std_wait::*, std_wtr_cycle::*, std_wtr_sector::*};
use crate::services::irrigation::states::{wiz_suspended::*, wiz_wait::*, wiz_wtr_cycle::*, wiz_wtr_sector::*};
use crate::services::irrigation::{cycle::*, wtr_engine::*};
use crate::{log_debug, logger::*};

use ctrl_prelude::{domain_types::*, string_resources::*};

pub trait EngineControl {
    fn exec_cmmnds(&mut self, time: CtrlTime);
    fn process_time_tick(&mut self, time: CtrlTime);
}

impl EngineControl for WtrEngine {
    #[inline]
    fn exec_cmmnds(&mut self, time: CtrlTime) {
        let mut next_state = self.wtr_cfg.state;
        let prev_state = next_state; // funciona porque o State é Copy
        let mut cmd: Command;
        let mut save_ret: State;
        // corre pelo menos uma vez quando chamado (assumindo que se passou um comando) e termina se não houver mudança de estado.
        // qd entra no loop entra com o estado atual e é o comando que fará ou não evoluir nos estados.
        while !self.cmd_queue.is_empty() {

            cmd = self.cmd_queue.pop_front().unwrap();

            loop {
                log_debug!(dbg_wtr_eng_curr_state(&self.wtr_cfg.state.to_string()));
                self.wtr_cfg.state = next_state;

                next_state = match self.wtr_cfg.state {
                    State::Starting => self.starting(time),
                    State::NoScheduleDef => self.no_schedule_def(&cmd),
                    State::EstablishMode => {
                        save_ret = self.establish_mode(&cmd, time);
                        cmd = Command::Null; //temos que "comer" aqui o comando, senão o change_mode nunca mais sai do estado
                        save_ret
                    }
                    State::ManWait => self.manual_wait(&cmd, time),
                    State::WzrWait => self.wizard_wait(&cmd, time),
                    State::StdWait => self.standard_wait(&cmd, time),
                    State::ManWtrCycle => self.manual_watering_cycle(&cmd, time),
                    State::StdWtrCycle => {
                        save_ret = self.standard_watering_cycle(&cmd, time);
                        cmd = Command::Null;
                        save_ret
                    },
                    State::WzrWtrCycle => self.wizard_watering_cycle(&cmd, time),
                    State::ManWtrSector => self.manual_watering_sector(&cmd, time),
                    State::StdWtrSector => self.standard_watering_sector(&cmd, time),
                    State::WzrWtrSector => self.wizard_watering_sector(&cmd, time),
                    State::ManWtrSectorDirect => self.manual_watering_sector_direct(&cmd, time),
                    State::SuspendedWizard => self.suspended_wizard(&cmd, time),
                    State::Error => self.error(&cmd, time),
                    State::Shutdown => self.shut_down(),
                };
                if self.wtr_cfg.state == next_state {
                    break; //não houve alteração do estado e já não temos comandos para processar
                }
                self.wtr_cfg.state = next_state;
            } //loop end
        } // while end
        self.wtr_cfg.save_if_updated(time);
        // advertising
        // Só comunicamos se houver alteração do estado.
        if next_state != prev_state || self.wtr_cfg.changed {
            self.msg_broker.snd_status_changed(time);
        }
    }

    #[inline]
    fn process_time_tick(&mut self, time: CtrlTime) {
        // é aqui que vamos avaliar se há algum evento cujo tempo para se ativar tenha chegado, e agir em conformidade
        // temos os estados da máquina, devemos utilizá-los para controlar as acções, e dar comandos á máquina com a info relevante
        // Ok, mas o cycle_start_notification, já tem uma guarda para o estado da máquina
        // o resto é necessário para house keeping dos .schedules (como por exemplo, não correndo, atualiza o tempo para o próximo evento)
        match self.wtr_cfg.state {
            State::Starting | State::NoScheduleDef | State::EstablishMode | State::Error | State::Shutdown | State::ManWait | State::ManWtrSectorDirect => (), //nop
            State::StdWait => {
                let mut o_ptr: Option<CYCLE_PTR> = None;
                let mut cycle: &mut Cycle;
                for ptr in self.std_ptrs.iter() {
                    cycle = &mut self.cycles[ptr.1 as usize];
                    if cycle.schedule.is_time_to_run(time) {
                        o_ptr = Some(ptr.1);
                        break; //apanhamos o primeiro que aparecer.  Se houver sobreposições, temos pena.
                    }
                }
                if let Some(ptr) = o_ptr {
                    // se houver sobreposição de ciclos por má definição., o motor não fará nada..escusava é de se perder tempo com isso se percebermos isso aqui.
                    // mas é sempre o exercicio de pôr o engine como foco do tratamento da lógica, ou distribuir essa lógica por muitos sitios.
                    self.snd_command(Command::StartCycle(ptr));
                }
            }
            State::WzrWait => {
                let cycle = &mut self.cycles[self.internal.wizard.unwrap() as usize];
                let cycle_ptr = cycle.ptr.unwrap();
                if cycle.schedule.is_time_to_run(time) {
                    self.snd_command(Command::StartCycle(cycle_ptr));
                }
            }
            State::WzrWtrCycle | State::StdWtrCycle | State::ManWtrCycle => {
                //senão, não há nenhum ciclo ativo (por exemplo por causa do recycle pump), e validamos se é altura de start do setor seguinte
                if let Some((sec_id, sec_ptr)) = self.wtr_secs.front() {
                    let running_ptr = RunningPtr {
                        cycle: self.active_ptrs.cycle,
                        sec_id: Some(*sec_id),
                        run_sec_ptr: Some(*sec_ptr),
                    };
                    let run_sector = self.run_secs.get(*sec_ptr as usize).unwrap();
                    if run_sector.start <= time {
                        self.active_ptrs = running_ptr;
                        self.snd_command(Command::StartSector)
                    }
                }
            }
            State::StdWtrSector | State::WzrWtrSector | State::ManWtrSector => {
                // só com um ciclo e setor ativo é que vale a pena validar o schedule dos setores para validar o termino
                let run_sector = self.run_secs.get(self.active_ptrs.run_sec_ptr.unwrap() as usize).unwrap();
                if run_sector.end <= time {
                    //se end < agora, de certeza que o start também já passou.
                    let c = self.active_ptrs.clone();
                    self.snd_command(Command::EndSector(c));
                }
            }
            State::SuspendedWizard => {
                // Analisa o estado de suspended e se é para retomar ou não
                if time >= self.suspend_timeout.unwrap() {
                    self.snd_command(Command::ResumeTimeOut);
                }
            }
        };
    }
}
