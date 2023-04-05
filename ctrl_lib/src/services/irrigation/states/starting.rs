use crate::data_structs::msgs::alert::*;
use crate::data_structs::rega::{state::*, wizard_info::*};
use crate::services::irrigation::{cycle_type::*, db_model::*, stress_ctrl::*, system_recover::*, wtr_engine::*};
use crate::{app_time::ctrl_time::*, config::wtr_cfg::*};
use crate::{log_info, logger::*};
use ctrl_prelude::{globals::*, string_resources::*};

pub trait Starting {
    fn starting(&mut self, time: CtrlTime) -> State;
}

impl Starting for WtrEngine {
    /// Restarts the machine, reload context state variables, etc.
    ///
    /// Machine first state
    ///
    /// Try to recover sector info, if for some reason last shutdown wasn't controlled
    ///
    ///
    ///
    /// Limpa bd, recupera do que houver para recuperar
    ///
    /// Estabelece as condições iniciais
    ///
    /// Faz a inicialização e verificação do sistema fisico
    ///
    /// Isto está num estado da máquina, em vez de na inicialização do objeto, para permitir o restart da máquina e limpeza das coisas.
    ///
    /// A idéia é se porventura a coisa "embrulhar", tentar desembrulhar com um restart da máquina remotamente, em vez de parar e rearrancar o programa.
    ///
    /// Claro que remotamente também deverá dar para aceder á máquina e fazer estas operações, mas se só tiver o telefone na mão (UI), será uma primeira
    /// tentativa com menos recursos (ainda não investiguei a instalação de um terminal no telefone para aceder á máquina)
    ///
    #[inline]
    fn starting(&mut self, time: CtrlTime) -> State {
        log_info!(INFO_PHISICAL_ADAPTER_INI);

        // Recuperação de situações estranhas
        system_check_and_recover(&self.db);

        let wc = &mut self.wtr_cfg;

        //reset/update values at machine start
        wc.in_alert = AlertType::NoAlert as u8;
        wc.in_error = 0;
        wc.changed = true;
        let mode = wc.mode;
        let opt_nanos_elapsed = Some(time.0 - wc.wizard_info.last_stress_control_time.0);

        // Baseline para o arranque.  Setores fisicos e ciclos de rega.
        self.cycles.clear();
        self.db.get_all_cycles(&mut self.cycles).unwrap();

        // Vamos criar os ciclos internos aqui
        // um ciclo manual permanente, never retry, para ser usado pelo modo manual sempre que necessário
        // um ciclo wizard
        // um ciclo wizard para eventual compensation se/qd necessário
        //
        // e simplificar a lógica de tratamento dos ciclos no momento em que se dá os comandos
        //
        // em tese fica mais rápido, porque esse trabalho é feito no arranque, e é menos uns ciclos de CPU nesse momento
        //
        // Isto são condições de preparação no arranque.
        match self.cycles.binary_search_by_key(&CycleType::Wizard, |v| v.cycle_type) {
            Ok(cycle_ptr) => self.internal.wizard = Some(cycle_ptr as u8),
            Err(new_pos) => self.internal.wizard = self.set_wizard_cycle(time, Some(new_pos)).unwrap(),
        };
        match self.cycles.binary_search_by_key(&CycleType::Direct, |v| v.cycle_type) {
            Ok(cycle_ptr) => self.internal.direct = Some(cycle_ptr as u8),
            Err(new_pos) => self.internal.direct = self.set_direct_cycle(time, Some(new_pos)).unwrap(),
        }
        // inicializa os ciclos, incluindo o schedule e os ptrs para os ciclos standard
        self.std_ptrs.clear(); //limpamos inicialmente porque isto pode ser chamada recorrentemente se houver um restart.
        let max = self.cycles.len();
        self.internal.have_standard = max > MAX_INTERNALS;
        let mut i = MAX_INTERNALS;
        while i < max {
            self.std_ptrs.push((self.cycles[i].run.cycle_id, i as u8));
            i += 1;
        }

        // Inicializa os ptr dos cycle
        for (ptr, cycle) in self.cycles.iter_mut().enumerate() {
            cycle.ptr = Some(ptr as u8);
        }

        // estabelecemos as condições de arranque
        establish_initial_conditions(&mut self.wtr_cfg, &mut self.sectors);
        self.wtr_cfg.fresh_start = 1; //deixa de ser fresh start - a não ser que se configure isso num qualquer restart - TODO - o restart ainda não está feito

        // atualizamos os niveis de stress desde a ultima paragem e arrancamos o timeout
        // cenários a considerar
        //   1. arranque pela primeira vez?  tem que se fazer setup diretamente na BD...ou o reset ds valores ao nivel no cliente
        //      trata disto.  Downside:  por erro limpa-se a coisa, e mexer na bd obriga a pensar o tema.
        //      Mas pode-se pôr uma "guarda" no reset para pedir confirmação
        //   2. arranque subsequentes: é só mesmo medir o tempo desde a ultima run porque em principio não passa mais do que alguns dias sem controlo
        //      e nesse cenário não justifica ser mais papista que o papa

        self.upd_deficit_and_stress_level(opt_nanos_elapsed, time);

        log_info!(info_starting_mode(&mode.to_string()));

        // se não estiver em modo standard, ou se houver um ciclo definido, para além dos ciclos internos
        if !mode.is_standard() || self.internal.have_standard {
            //  avançamos para o estabelecimento do modo e estado seguinte
            State::EstablishMode
        } else {
            // senão  não á condições de avançar
            State::NoScheduleDef
        }
    }
}

#[inline]
pub fn establish_initial_conditions(wtr_cfg: &mut WtrCfg, sectors_list: &mut SecList) {
    // Condições iniciais para o wizard da máquina de rega
    //
    // - A documentação associada a jardins indica 25 mm por semana, portanto a cada sete dias.
    // - Indicam também que se deve regar profundamente e dia sim/dia não, mas aqui pelas contas que fiz para o meu tipo de terreno, dado o tipo de solo (areia)
    //   isto não parece ser suficiente.  Sobre este particular, tinha a hipótese de desenhar algo generérico para gerir o tipo de terreno, mas decidi
    //   simplificar, e jogar com os parametros mais básicos, simplificando a lógica da aplicação.
    //
    // - Talvez no futuro depois de esta primeira versão estar estável melhore isto. SPRINT: FUTURE
    //
    // - Por outro lado a documentação associada á agricultura indica algo parecido, mas sem referência a uma quantidade máxima
    //   Utiliza outro referencial, nomeadamente que a necessidade de água das plantas é:
    //      - estabelecer a SATuração do terreno e o WL - water level para o tipo de cultura - e isto bate certo com a praxis que verifiquei durante a planatação da relva
    //        onde se teve que manter todos os dias as raizes da relva humidas (com água) para as plantas "agarrarem" e terem nutrientes para crescer
    //      - e a partir daqui o fator de orientação é o ET da planta, e é esse que define o INtake de água da planta.
    //        Para a relva este numero (de tabela) é entre 4 a 6 mm /dia portanto está próximo do valor referido acima, mais para próximo dos 4
    //      - Assim a quantidade de água a regar por dia deverá ser, assumindo que SAT e o WL foram estabelecidos no momento zero
    //        (ver excel onde seria necessário regar 6 dias seguidos, 2 vezes / dia, durante +- 30'):
    //
    //   ET (desde a ultima vez que se regou) - Chuva (desde a ultima vez que se regou) + Percolation( que varia por setor mas é +- 6.985 /dia)
    //
    //   Voltando aos requisitos.
    //
    //   Na entrada em produção do sistema, vamos assumir (definido como requisito base) que se faz a rega profunda que for necessária para se estabelecer
    //   a impregnação total do terreno (estabelecer o nivel da agua - water level)
    //
    //   Depois daí em diante é manter de acordo com a fórmula acima.
    //   Mas ainda assim, há que prevêr situações de avarias ou imprevistos, que num contexto remoto,
    //   facilmente podem deixar o terreno 1 a 2 semanas sem regar de forma consistente
    //   (liga desliga da máquina enquanto se resolve o tema que tenha ocorrido)
    //
    //   Portanto no arranque temos que ir perceber em que situção se está:
    //      - Entrada em produção
    //      - Rearranque depois de uma qualquer paragem (manutenção sistema, avarias, etc.)
    //
    //
    //   Uma decisão é o que fazer após um tempo relativamente longo de paragem:
    //      A - assumir como um fresh start (como se fosse pela primeira vez)
    //      B - ou assumir algo diferente?
    //
    //  Tomando com referencial para o nível da água os 150mm das raizes , valor médio deste tipo de cultura - relva, e o percolation de 0,001666667 mm/min
    //  teriamos que ao fim de 62 dias +- 2 meses (root distance / percolation -> converted to days) o WL está abaixo da raiz.
    //  (isto parece baixo, mas pressupoe que o WL foi adequadamente estabelecido inicialmente - na realidade seria menos tempo, porque a percolation
    //  varia com a quantidade de água no terreno, e é baixa quando o terreno está impregnado - conforme vai secando, a percolation vai aumentando portanto
    //  contas dedo no ar, diria que o tempo é á vontade metade disto - 1 mês - uma formula empirica que estudei, e que deverá estar aproximado é
    //  que a velocidade de percolation será aproximada a  nr_dias_sem_rega * percolation_n-1 * exp(nr_dias_sem_rega / 209 , com o percolation_n-1 = ao percolation
    //  do terreno saturado referido acima no dia 1 )
    //
    //  Por esta conversa toda se percebe o problema filosófico de fundo:
    //  1. A relva precisa de agua como ó caraças
    //  2. Ao fim de 1 mês sem regar (cenário que deve acontecer muito pouco) ter-se-ia que reestabelecer o WL
    //     (rega profunda 2 vezes / dia durante uma semana pelas contas de merceeiro)
    //  3. Cada paragem de manutenção (1 mês é pouco provável, mas 2 semanas é fácil de acontecer....) obriga a um cenário próximo do ponto 2.
    //  4. e como deviamos fazer mnt 1 vez por ano , quer dizer que teriamos um ciclo profundo no inicio da primavera ou inicio do verão...
    //  5. mas que nos diz que o modo wizard coloca a máquina drurante muito tempo na "capacidade" máxima
    //  6. o que leva a questionar a sua utilidade (poupança de energia), uma vez que mantendo-se só o modo standard simplifica o programa ...
    //
    //  Portanto, no que ficamos?  O que decidir?
    //
    //  Avançar com o modo wizard ou não, sabendo que a "poupança e 'inteligencia'" da coisa implica que ou é utilizado de forma continua, ou então os
    //  ganhos potenciais não se vão verificar na prática...
    //
    //  Mas pronto, como o objetivo é didático, e por outro lado é suposto ser utilizado de forma contínua e minimizando as paragens
    //  vamos avançar com o modo wizard.
    //
    //  Assim, vamos por a máquina a fazer as contas para autocompensar o que tiver que autocompensar.
    //
    //  Então temos que perceber quando parou da ultima vez para estabeler as condições iniciais do deficit:
    //
    //  1. deficit = 0 no fresh start - pressupõe deep water e estabelecimento do water level - hard requirement
    //  2. fazer as contas daí para a frente, contas que serão:
    //      a. calcular o deficit para o nr de dias em que se parou, tendo em atenção a chuva e o percolation no periodo
    //      b. limitar o deficit ao comprimento das raizes da relva (150mm - que é a+- ao fim de 15 dias)
    //
    // - Ter em atenção o entrar ou não em stress (falta de água)
    //      - choque de calor
    //      - se choveu
    //      - evapotranspiração das plantas
    //      - precipitação no solo em função da permeabilidade (este fator não estou a considerar - ver notas)
    // - máximo tempo de rega para não "cansar" o motor de rega
    //
    // ou seja
    //
    // - tentar regar o máximo evitando o runoff
    // - no controlo do stress calculamos a que distânia estamos do target
    // - aqui vamos calcular o tempo de rega para repor o nivel no sitio certo considerando:
    //     - quantidade de água calculada
    //     - rega pelo máximo evitando o runoff
    // - têm por fim uma guarda de segurança para evitar regas abaixo de x segundos (defini 60 segundos),
    //   para evitar ligar e desligar o motor por periodos que são inconsequentes, que só gasta energia, com pouco efeito prático nas plantas e desgasta o material
    //
    if wtr_cfg.fresh_start == 0 || wtr_cfg.last_stop > wtr_cfg.live_since {
        // é fresh start, quando é mesmo um fresh start, ou se a data de paragem for maior do que a data do live since, que ou o tempo andou para trás
        // ou é nas situações de testes ou simulação onde isso pode acontecer.
        // Nestes casos assume-se fresh start.  Situações de testes ou simulação não é relevante ter os dados certos.
        wtr_cfg.last_stop = wtr_cfg.live_since;
    }
    // no fresh start nr_of int days será igual a zero porque ou o live_since é igual ao last_stop, e dá zero,
    // ou estão muito próximos, o que dará zero também porque estamos a fazer uma divisão inteira.
    // assert!(wtr_cfg.live_since >= wtr_cfg.last_stop, "live since should be after last_stop!");  // ver comentário no if acima
    let nr_of_int_days_stopped = ((wtr_cfg.live_since.sod_ux_e().0 - wtr_cfg.last_stop.sod_ux_e().0) / CtrlTime::NR_NANOS_IN_A_DAY) as f32;

    let rain = 0.; //No SPRINT weather podemos pensar em procurar fontes de informação adicionais para tapar a falta de info

    let mut percolation = 0.;
    let mut et = 0.;
    for sec in sectors_list.iter_mut() {
        if nr_of_int_days_stopped >= 1. {
            // já passou pelo menos um dia inteiro e pela mecãnica das variaveis e máquina, não é garantidamente um fresh start
            // o valor configurado no setor são mm por hora
            percolation = nr_of_int_days_stopped * 24. * sec.percolation;
            et = nr_of_int_days_stopped * wtr_cfg.wizard_info.daily_tgt_grass_et;
        } else {
            // se for fresh_start estamos a arrancar com o WL e SAT definidos, pelo que o et = 0 e o percolation = 0 (por definição dos requisitos)
            if wtr_cfg.fresh_start == 0 {
                sec.last_watered_in = CtrlTime(0);
            }
            // senão
            // não é um fresh start  o que quer dizer que arrancou e parou, mas não passou um dia
            // e se não passou um dia, a mecanica geral de ir buscar os valores do dia anterior deverá funcionar...
            // o único tema é que se a máquina parou, não há info de chuva, pelo que se poderá pensar num SPRINT para complementar esta info de outra fonte
        }
        sec.deficit = (sec.deficit + percolation - rain + et).clamp(-1., GRASS_ROOT_LENGTH);
        sec.update_db = true;
    }
}
