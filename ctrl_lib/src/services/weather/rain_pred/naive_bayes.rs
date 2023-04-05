// A ideia e pipeline geral é a que está abaixo
//
// Estou a assumir, educated guess, que o importante é o referencial dia.
//
//
// Processo Nivel 1
//  0. Existe uma preparação manual de histórico para o startup do modelo, que é importado para as tabelas.  A partir desta primeira vez, é automático.
//      Esta preparação manual trata de eventuais dados em falta resolvendo-os por extrapolação linear e agrega logo os dados por dia (tenho um modelo de excel para isso)
//      A preparação manual analisa os dados, resolve falhas, e calcula os agregados diários/horários para as necessidades do modelo
//      O próprio modelo é calculado "offline", e carregado na tabela.
//      O programa apenas lê os dados do modelo e calcula o que tiver que calcular.
//
// NOTA IMPORTANTE - este é um modelo especializado que foi otimizado tentando evitar as multiplas passagens por n vetores, que são normais num modelo mais generalista
// ou biblioteca que se pretende reutilizar em diferentes contextos.
// Portanto a informação de histórico para o arranque do modelo foi preparada antes manualmente, e dai em diante o programa assegura que a informação é mantida
// sabendo-se que pode haver paragens e rearranques, mas o modelo naive bayes é resiliente a falhas de informação e trabalha bem como poucos dados.
// Aliás, uma coisa a testar, é a evolução da precisão do forecast ao longo do tempo e em diferentes cenários de quantidade de dados de treino do modelo

// DESIGN DECISION
// O código rust do modelo está no módulo dos testes, porque a ideia é ler sempre da BD o modelo definido.
// como o modelo é supervisado, e deve ser acompanhado para avaliar se está a desempenhar como previsto ou não, decidi não andar a treinar o modelo ás cegas
// Desta forma o trabalho será feito todo offline, usando as funções de teste, e em produção o código apenas carrega o modelo ativo e usa-o para calculo da previsão
// O trabalho de mastigar dados e treinar e seleccionar o melhor modelo será offline
// A opção de fazer isso automaticamente é uma possibilidade, mas não vi grande valor acrescentado nisso, face ao tempo que pode demorar a treinar o modelo,
// coisa que só percebi depois de fazer quase tudo, e de perceber que existe uma grande variabilidade na eficiência dependendo da aleatoriadade da escolha dos dados para treino
// O trabalho de monitorização será assim, de quando em vez (algumas vezes por ano - mais intenso até ter dados da estação local a cobrir as necessidades)
// avaliar os dados que existem de previsão (comparar o previsto com o real) e
// decidir se há que retreinar o modelo ou não, caso identifique que a eficiência real tem um desvio significativo da eficiência prevista.
// Em tese é só retreinar, porque os dados diários, esses serão mantidos na tabela ml_raw_data

use std::f64::consts::PI;

use crate::data_structs::sensor::stat_metric::*;
use crate::services::weather::db_model::*;
use crate::services::weather::rain_pred::data_structs::*;
use crate::{db::db_sql_lite::*, lib_serde::*};
use crate::{log_warn, logger::*};

#[inline]
pub fn get_model(db: &Persist, curr_model: u32) -> Option<Model> {
    if let Ok(Some(model_str)) = db.get_model(curr_model) {
        match data_from_str::<Model>(&model_str) {
            Ok(m) => Some(m),

            Err(err) => {
                log_warn!(ConversionError::DeserializationIssue(model_str, err.to_string()));
                None
            }
        }
    } else {
        warn!("Não foi encontrado o modelo de previsão nr: {}", curr_model);
        None
    }
}

// Por um lado podiamos carregar o modelo no arranque e evitava-se um roundtrip e deserialização todos os dias
// Por outro, Se alterarmos ou criarmos um novo modelo, para além de se alterar na BD, se estiver em memória têm-se também que alterar na memória => + complexidade na lógica
#[inline]
pub fn get_rain_probability(db: &Persist, x_vec: &DSRow<MAX_FEATURES>, curr_model: u32) -> Option<NBGaussianPrediction> {
    get_model(db, curr_model).map(|model| predict_probability(&model, x_vec))
}

#[inline]
pub fn log_likelihood(model: &Model, x: &[f64; MAX_FEATURES]) -> [f64; NR_CLASSES] {
    let nr_features = model.selected_features.len();
    let mut jll: [f64; NR_CLASSES] = [0.; NR_CLASSES];
    let mut jointi: f64;
    let mut sum1: f64;
    let mut sum2: f64;
    let mut n_ij: f64;
    let mut welford: &WelfordMeanAndVar;
    let mut col: usize;

    let mut aux_var: f64;
    let mut aux_ln: f64;
    for (class, jll_class_val) in jll.iter_mut().enumerate() {
        aux_ln = model.class_prior[class];
        assert!(aux_ln > 0.);
        jointi = f64::ln(aux_ln);
        sum1 = 0.;
        sum2 = 0.;
        // para cada uma das features seleccionadas vamos fazer o fit
        for feature in 0..nr_features {
            col = model.selected_features[feature];
            welford = &model.fit_stats[class][col];
            aux_var = welford.var_sample(); // + VAR_SMOOTHING * model.max_var;
            if aux_var == 0. {
                // isto é necessário para evitar a div por zero - side effect - a eficiência do modelo melhora...matematicamente não sei porquê
                // para registo, apenhei entretanto rapaziada que utiliza o VAR SMOOTHING como hyper parameter para melhorar o modelo, e ensaia
                // diferentes valores para melhorar a eficiência.  Como já estamos nos 99% de F1 Score, não faz sentido investir ai mais tempo
                // TODO vamos por isto "live" e medir o desempenho real no terreno para perceber se é necessário fazer alguma afinação
                aux_var = 1e-300_f64;
            }
            sum1 += f64::ln(2. * PI * aux_var);
            sum2 += f64::powf(x[col] - welford.mean, 2.) / aux_var;
        }
        n_ij = -0.5 * sum1;
        n_ij -= 0.5 * sum2;
        *jll_class_val = jointi + n_ij;
    }
    jll
}

// O tema do max na função é para estabilizar numericamente as contas.
// subtrai-se o maior valor, para que a soma dos expoentes seja sobre numeros pequenos para reduzir a probabilidade de overflows.
// É por isso que os dados em machine learning devem estar normalizados ou standardizados.  Para evitar trabalhar com numeros grandes que não ajudam á coisa.
// Na net isto é conhecido como o log sum trick
#[inline]
pub fn log_sum_exp(x: &[f64], x_max: f64) -> f64 {
    let max = if !x_max.is_finite() { 0. } else { x_max };
    let mut sum = 0.;
    for f in x.iter() {
        sum += f64::exp(*f - max);
    }
    sum = f64::ln(sum);
    sum += max;
    sum
}

#[inline]
pub fn predict_log_probability(model: &Model, x: &DSRow<MAX_FEATURES>) -> NBGaussianPrediction {
    let mut prediction = predict(model, x);
    let log_prob_x = log_sum_exp(&prediction.jll, prediction.get_max_jll());
    for (idx, f) in prediction.log_probability.iter_mut().enumerate() {
        *f = prediction.jll[idx] - log_prob_x;
    }
    prediction
}

#[inline]
pub fn predict_probability(model: &Model, x: &DSRow<MAX_FEATURES>) -> NBGaussianPrediction {
    let mut prediction = predict_log_probability(model, x);
    for (idx, f) in prediction.log_probability.iter().enumerate() {
        prediction.probability[idx] = f64::exp(*f);
    }
    prediction
}

#[inline]
pub fn predict(model: &Model, x: &DSRow<MAX_FEATURES>) -> NBGaussianPrediction {
    let mut prediction = NBGaussianPrediction {
        jll: log_likelihood(model, x),
        ..Default::default()
    };
    update_max(&mut prediction);
    prediction
}

#[inline]
pub fn update_max(pred: &mut NBGaussianPrediction) {
    let idx = max_index(&pred.jll);
    pred.index = idx;
}
