#[allow(unused_imports)]
use crate::app_time::{ctrl_time::*, tm::*};
use crate::config::wtr_cfg::*;
use crate::data_structs::rega::wizard_info::GRASS_ROOT_LENGTH;
use crate::services::{irrigation::sector::*, weather::db_model::*};
use crate::{db::db_sql_lite::*, log_warn, logger::*};

/// Este é um valor indicativo aproximado para o estado da coisa
///
/// A ideia simples é:
/// - tendo como referencial o valor esperado para o dia da semana em que se está,
/// - ver onde estamos realmente
///
/// Com o seguinte score:
///  'Alerta'       |   'dominio'       | 'resultado'
/// ----------------|-------------------|-------------
///  Emergency,     | if score < 50.%   |    => 3,
///  Alert,         | if (50. ..90.)    |    => 2,   
///  Normal,        | if (90. ..110.)   |    => 1,   
///  Over,          | irrigated else    |    => 0,  
#[inline]
pub fn calc_sec_stress_score(deficit: f32) -> (u8, f32) {
    // considerando o debit e a rega total - daqui para baixo não interessa controlar
    let stress_perc = ((1. - ((GRASS_ROOT_LENGTH - deficit) / GRASS_ROOT_LENGTH)) * 100.).clamp(0., 100.); //.max(minimum_reference);
    let score: u8 = match stress_perc {
        stress_perc if stress_perc < 20. => 3,                   // Emergency
        stress_perc if (20. ..60.).contains(&stress_perc) => 2,  // Alert
        stress_perc if (60. ..100.).contains(&stress_perc) => 1, // Normal
        _ => 0,                                                  // over irrigated
    };
    (score, stress_perc)
}

/// return (minutes, deficit)
/// <br>
/// Chamamos sempre o wizard, porque temos que manter sempre o controlo para ter a informação tão atualizada quanto possivel se houver uma mudança de modo
#[inline]
pub fn dur_man_strategy(is_manual: bool, db: &Persist, sec: &Sector, wtr_cfg: &WtrCfg, time_now: CtrlTime) -> (f32, f32) {
    let (_duration, new_deficit) = dur_wzrd_strategy(db, sec, wtr_cfg, time_now);
    if !is_manual {
        (sec.max_duration, new_deficit)
    } else {
        (Sector::MAX_SECTOR_WORK_MINUTES, new_deficit)
    }
}

/// return (minutes, deficit)
/// <br>
/// Chamamos sempre o wizard, porque temos que manter sempre o controlo para ter a informação tão atualizada quanto possivel se houver uma mudança de modo
#[inline]
pub fn dur_std_strategy(db: &Persist, sec: &Sector, wtr_cfg: &WtrCfg, time_now: CtrlTime) -> (f32, f32) {
    let (_duration, new_deficit) = dur_wzrd_strategy(db, sec, wtr_cfg, time_now);
    (sec.max_duration, new_deficit)
}

/// - A documentação associada a jardins indica 25 mm por semana, portanto a cada sete dias.
/// - Indicam também que se deve regar profundamente e dia sim/dia não.
/// - Ter em atenção o entrar ou não em stress (falta de água)
///      - choque de calor
///      - se choveu
///      - evapotranspiração das plantas
///      - precipitação no solo em função da permeabilidade (este fator não estou a considerar - ver notas)
/// - máximo tempo de rega para não "cansar" o motor de rega
///
/// Por outro lado a documentação associada á agricultura indica algo parecido, mas sem referencia a uma quantidade máxima
///
/// Utiliza outro referencial, nomeadamente que a necessidade de agua das plantas é:
/// - estabelecer a SATuração do terreno e o WL - water level para o tipo de cultura - e isto bate certi com a praxis que verifiquei durante a planatação da relva
///   onde se teve que manter todos os dias as raizes da relva humidas (com água) para as plantas "agarrarem" e trrem nutrientes para crescer
/// - e a partir daqui o fator de orientação é o ET da planta, e é esse que define o INtake de água da planta.  
///   Para relva este numero (de tabela) é entre 4 a 6 mm /dia portanto está próximo do valor referido acima, mais para próximo dos 4
/// - Assim a quantidade de água a regar por dia deverá ser
///   (assumindo que SAT e EL foram estabelecidos no momento zero = caso não fossem ver excel onde seria necessário regar 6 dias seguidos, 2 vezes / dia, durante +- 30'):
///
///   ET (desde a ultima vez que se regou) - Chuva (desde a ultima vez que se regou) + Percolation( que varia por setor mas é +- 6.985)
///
/// ou seja
///
/// - tentar regar o máximo evitando o runoff
/// - no controlo do stress calculamos a que distância estamos do target
/// - aqui vamos calcular o tempo de rega para repor o nivel no sitio certo considerando:
///     - quantidade de água para a semana,
///     - rega pelo máximo evitando o runoff
/// - têm por fim uma guarda de segurança para evitar regas abaixo de x segundos (defini 60 segundos),
///   para evitar ligar e desligar o motor por periodos que são inconsequentes, que só gasta energia,
///   com pouco efeito prático nas plantas e desgasta o material
///
/// Como já se atualizou o deficit dos setores pelo controlo de stress, aqui só precisamos de ir buscar o eT á BD
///
/// return (minutes, deficit)
///
/// O deficit devolvido, deverá ser usado para ajustar o deficit do setor em causa.  
/// Só não atualizo logo aqui para manter separação de responsabilidades, e manter a função pura, para manter os pontos de alteração de dados
/// em numero estritamente necessário, o que facilita os testes e a fiabilidade
#[inline]
pub fn dur_wzrd_strategy(db: &Persist, sec: &Sector, wtr_cfg: &WtrCfg, time_now: CtrlTime) -> (f32, f32) {
    let mut percolation: f32 = 0.;
    let mut et = wtr_cfg.wizard_info.daily_tgt_grass_et; // assumimos por missão, em caso de falta de dados, o valor médio definido pelos expertos

    // se o setor já foi alguma vez regado, vai-se ver a precipitação da agua no terreno
    // O starting já atualizou o deficit com a percolation até ao ultimo dia no caso de pára-arranca
    // aqui parece estar a faltar o ultimo "bocado" desde o ultimo dia interiro do starting, no caso se rearranque
    if sec.last_watered_in.0 > 0 {
        // se o setor já foi alguma vez regado, vai-se ver a precipitação da agua no terreno
        assert!(time_now.0 >= sec.last_watered_in.0, "Current time must be greater than the last watered time.");
        percolation = nano_to_min(time_now.0 - sec.last_watered_in.0) * (sec.percolation / 60.);
        // mas temos aqui que analisar se houve paragem da maquina
        // se a máquina esteve parada, o Starting já atualizou o deficit dos setores, pelo que não entra no if
        // se é fresh start o et é zero
        if sec.last_watered_in >= wtr_cfg.live_since {
            //quer dizer que não houve paragem
            let o_et: Option<f32> = db.get_et_between(sec.last_watered_in.sod_ux_e(), time_now.sod_ux_e()); // et desde a ultima vez que regou
            if let Some(_et) = o_et {
                if (_et - 0.).abs() > 0. {
                    // se o valor vier a zero, é porque não há dados e nesse caso assume-se o default logo na definição da variável
                    et = _et;
                }
            } else {
                log_warn!("Falta a informação da eT para o calculo do tempo de rega.");
            }
            // não vamos buscar a chuva, porque a dita está a atualizar periodicamente os setores no stress control
        }
    }
    // else {
    //     // estamos a ligar a maquina pela primeira vez
    //     // assume que o terreno foi regado pela saturação e o water level está no sitio certo, e a percolation é zero
    //     // tudo o que está na definição das variáveis acima
    // }

    // ter em atenção que o eT é uma formula empirica, apesar de recomendada.  Falta aqui um fator para o eT, mas isso fica para um SPRINT posterior.
    // onde eventualmente ponhamos isso num parametro da BD para irmos afinando ao longo do tempo
    let mut new_deficit: f32 = et + percolation + sec.deficit; // a formula base subtrai a chuva, mas já fazemos isso no stress control que atualiza o deficit

    // calculamos o ceiling porque desde que se liga a valvula até a agua chegar aos sprinklers com o debito nominal, vão uns bons segundos
    // como isso é muito dificil de determinar com exatidão (eventualmente medir e calcular o "fator certo")
    // esta é uma aproximação grosseira para compensar esse fator, por agora
    let mut duration = f32::ceil(new_deficit / sec.debit);
    duration = apply_duration_rules(duration, wtr_cfg);
    // Negative new_deficit values means runoff, so, lets say that everything < -1 mm will go somewhere, but it will not be "stored"
    // Values > GRASS_ROOT_LENGTH we assume that are not available anymore for the grass to get (bellow root length)
    new_deficit = new_deficit.clamp(-1., GRASS_ROOT_LENGTH);
    (duration, new_deficit)
}

#[inline]
fn apply_duration_rules(duration: f32, wtr_cfg: &WtrCfg) -> f32 {
    if duration <= 1. {
        // safeguard for practical inconsequent watering , i.e.,considering ,
        // we would have a few seconds to start the pump + a few seconds to have the tank pressured + a few seconds to have the water arrive to the sprinkler
        // + a few seconds for the sprinklers to get to nominal debit
        // with very few practical effects considering actual watering time
        // não regamos, mas os pozinhos que era para regar são acumulados no deficit
        // Negative values are also accumulated (up to 1mm.  note bellow), but naturally, duration will be zero
        0.
    } else if duration > wtr_cfg.max_sector_time as f32 {
        wtr_cfg.max_sector_time as f32
    } else {
        duration
    }
}

/// O deficit devolvido, deverá ser usado para ajustar o deficit do setor em causa.  
/// Só não atualizo logo aqui para manter separação de responsabilidades, e manter a função pura, para manter os pontos de alteração de dados
/// em numero estritamente necessário, o que facilita os testes e a fiabilidade
#[inline]
pub fn dur_wzrd_strategy_for_resume(sec: &Sector, wtr_cfg: &WtrCfg, time_now: CtrlTime, cycle_start: CtrlTime) -> (f32, f32) {
    assert!(time_now.0 >= cycle_start.0, "Current time must be greater than the last watered time.");
    // no resume só interessa o percolation desde o arranque o ciclo, que foi o tempo utilizado para calcular o percolation antes do suspend
    let percolation = nano_to_min(time_now.0 - cycle_start.0) * (sec.percolation / 60.);
    let mut new_deficit: f32 = percolation + sec.deficit;

    // calculamos o ceiling porque desde que se liga a valvula até a agua chegar aos sprinklers com o debito nominal, vão uns bons segundos
    // como isso é muito dificil de determinar com exatidão (eventualmente medir e calcular o "fator certo")
    // esta é uma aproximação grosseira para compensar esse fator, por agora
    let mut duration = f32::ceil(new_deficit / sec.debit);
    duration = apply_duration_rules(duration, wtr_cfg);
    // Negative new_deficit values means runoff, so, lets say that everything < -1 mm will go somewhere, but it will not be "stored"
    // Values > GRASS_ROOT_LENGTH we assume that are not available anymore for the grass to get (bellow root length)
    new_deficit = new_deficit.clamp(-1., GRASS_ROOT_LENGTH);
    (duration, new_deficit)
}

#[cfg(test)]
mod tests {
    use crate::services::irrigation::wzrd_algorithms::calc_sec_stress_score;

    #[test]
    pub fn test_calc_stress_score() {
        let mut deficit: f32 = 0.;

        println!("dia 1 antes de regar");
        let daily_tgt_grass = 3.571428571;
        println!("score={:?}", calc_sec_stress_score(deficit));
        println!("dia 1 depois de regar");
        deficit = 0.;
        println!("score={:?}", calc_sec_stress_score(deficit));

        deficit = 0.;
        for day in 0..7 {
            println!("dia {} antes de regar", day);
            println!("score={:?}", calc_sec_stress_score(deficit));
            deficit += daily_tgt_grass;
            println!("dia {} depois de regar", day);
            println!("score={:?}", calc_sec_stress_score(deficit));
        }
    }
}
