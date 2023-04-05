use ctrl_prelude::error::build_error;

use crate::app_time::{ctrl_time::*, tm::*};
use crate::services::irrigation::{wtr_engine::*, wzrd_algorithms::*};
use crate::services::weather::db_model::*;
use crate::{log_error, logger::*};

pub trait StressControl {
    fn upd_deficit_and_stress_level(&mut self, nanos_elapsed: Option<u64>, time_ref: CtrlTime);
    fn stress_process_time_tick(&mut self, time: CtrlTime);
}

impl StressControl for WtrEngine {
    #[inline]
    fn stress_process_time_tick(&mut self, time: CtrlTime) {
        // Executa sempre a avaliação do stress do jardim, periodicamente de acordo com o schedule
        if self.stress_control_sched.is_time_to_run(time) {
            self.upd_deficit_and_stress_level(None, time);
            _ = self.stress_control_sched.set_next_event().map_err(|e| log_error!(build_error(&e)));
        }
    }
    ///
    /// Chamada a cada  stress_control_interval  minutos (definido como 6 minutos em 29/Jan/2020 - para facilitar as contas = 10 vezes / min.),
    /// e no arranque para determinar o saldo de água (- água perdida + chuva ) desde a ultima execução
    ///
    /// Controlamos onde está o nivel da água - isto está indiretamente relacionado com a distância em função da
    /// velocidade de precipitação no terreno e do tipo de solo
    ///
    /// - param: Option<seconds_elapsed>
    /// - param: time = tempo no momento da chamada
    /// - return: ()
    ///
    /// Se nanos_elapsed fôr None, assume-se que o tempo que passou é o intervalo definido na configuração do wizard_config
    /// Senão, utiliza-se esse tempo como a base para calcular o resultado
    ///
    #[inline]
    fn upd_deficit_and_stress_level(&mut self, nanos_elapsed: Option<u64>, time: CtrlTime) {
        let wi = &self.wtr_cfg.wizard_info;
        let db = &self.db;
        let sectors_list = &mut self.sectors;
        let nanos_elapsed: u64 = nanos_elapsed.map_or_else(|| min_to_nano(wi.stress_control_interval as f32), |value| value);

        // vamos calcular a partir de quando vamos buscar a chuva.
        //
        // se chover a cantaros, a água pode levar algum tempo a escoar, caso a coisa esteja mesmo alagada....mas a maquina compensará não regando
        //
        // Alagado implica ter o terreno cheio de agua até ao lençol friático a 2,5 m
        // e a agua sai seja por escorrer para as bermas (aquilo nunca alagou, pelo que em tese o escoamento está ok)
        // seja pela evapotranspiração (no inverno/primavera/outono que são os meses de maior probabiliade de chuva, o eT é menor portanto...1 a 3 mm dia)
        // e a velocidade de precipitação será baixa, uma vez que o terreno está impregnado, portanto...1 mm /hora, ou seja 24 mm/ dia
        // portanto em tese, desde que para de chover, a agua vai escoando á velocidade da percolation, que vai abaixo da raiz em 15 dias
        // ou seja...
        //
        // começando como "alagado", e considerando que a relva tem raizes até 150 mm, ao fim de 2,16 dias já se poderia em tese recomeçar a regar
        //
        // seguindo esta racional, "alagado" significa chuva que faça o mesmo que SAT + WL
        //
        // o controlo diário deve conduzir a este acerto "automaticamente", na medida em que se a "quota" do ciclo está preenchida, salta para o dia seguinte.
        // senão rega pela diferença

        // SPRINT WEATHER2 - Outras realidades podem recomendar fazer isto de forma mais inteligente e em função dos valores de precipitação de cada setor
        // Se a máquina parou mais tempo, quer dizer que os valores do deficit não serão os reais, 
        // mas o principio da maquina é estar a funcionar em continuo, pelo que nessa situação, os valores convergem para os valores corretos.
        // situações diferentes, como sejam paragens que somem tempos de ausencia de medição curtos ou longos
        //    - nos periodos de paragem curtos a diferença será pequena, e converge no tempo
        //    - nos periodos de paragem longos, outros temas quaisquer serão mais importantes, e a prórpia atualização não refleteria a realidade porque também se terá que medir a chuva
        // Enfim, manter o deficit atualizado é um tema que requer uma análise mais detalhada e para já arrancamos assim
        let rain_since = (time - nanos_elapsed).min(time - CtrlTime::NR_NANOS_IN_A_DAY);

        let rain_result = db.get_rain_between(rain_since, time);
        let mut score: u8;
        let mut stress_perc: f32;
        if let Some(rain) = rain_result {
            for sec in sectors_list.iter_mut() {
                sec.deficit = (sec.deficit - rain).max(-1.); //inferior a zero quer dizer que houve runoff
                (score, stress_perc) = calc_sec_stress_score(sec.deficit);
                sec.stress_perc = stress_perc;
                sec.stress_score = score;
            }
            _ = self.save_secs(); //O save_secs já loga o error

            // SPRINT WATER wizard - Requirements
            // emergency situations (heat strokes, others to identify), maybe a deep watering cycle
        }
    }
}
