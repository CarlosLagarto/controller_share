//  1. Le dados origem
//      1.1 vai buscar os dados á tabela dos sensores e contruir o data set base
//          o processo de fim de dia agrega dados pelas horas na sensor_daily_data, assegurando que mantemos o conjunto de dados necessário para
//          ter o data set balanceado entre as classes definidas. (o resto está em histórico)
//          O processo de carregamento/manutenção na ml_raw_data é manual
//
//          O processo de fim de dia é que trata de criar as méticas diárias e horárias.
//
//      1.2 se faltarem dados, têm que se resolver manualmente.
//  2. Prepara dados, prepara estatisticas fase seguinte - agregação por dia, que é o que defini para o modelo
//      2.1 Prepara dados - cria data set
//          validar se existem os dados necessarios para o modelo nas tabelas associadas á monitorização do tempo, nomeadamente
//      2.3 trabalho estatistico
//          - calcular todas as estatisticas ou auxiliares necessárias para o modelo e para a seleção das features relevantes para o modelo
//          - pelo coeficiente de correlação de pearson, seleccionar as variaveis com correlação < 0.95
//  3. Faz o fit do modelo
//      3.1 se as estatisticas já existirem todas da fase anterior, aqui será só aplicar as formulas sem necessidde de passagem pelos dados
//  4. calcular forecast
//      4.1 e mais uma vez aqui será só aplicar as formulas e calcular a probabiliade resultante

use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use rand::thread_rng;

use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::data_structs::sensor::stat_metric::*;
use ctrl_lib::services::weather::rain_pred::{data_structs::*, naive_bayes::*};
use ctrl_lib::{db::db_sql_lite::*, lib_serde::*, logger::*};

use crate::integration::naive_bayes::code::{data_structs::*, db_model::*};

// aqui vamos fazer o resampling para equilibrar o numero de registos de cada class no dataset, para balancear o algoritmo naive bayes
// para depois analisar a correlação entre todas as variaveis - todas as combinações 2 a 2 - para identificar aquelas que estão corr > 0.95
// e escolher as features para a análise dinamicamente
#[inline]
pub fn prep_data(ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>, model: &mut Model) {
    oversample_if_needed_and_correlate(model, ds);
    standardize(ds, model);
}

#[inline]
pub fn standardize(ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>, model: &mut Model) {
    // agora tratamos das basic stats á cabeça para não andarmos a fazer a conta em cada linha - faz-se só uma vez.
    for col in 0..ds.cols {
        ds.basic_stats[col].mean = ds.pearson_correlation[col][col].mean();
        ds.basic_stats[col].var = ds.pearson_correlation[col][col].var();
        ds.basic_stats[col].sigma = ds.pearson_correlation[col][col].sigma();
    }
    // E agora vamos standardizar, para o fit
    // e agora temos que ir analisar a correlação das colunas na matriz para perceber quais as colunas que vamos deixar cair por terem correlação > 0.95
    ds.select_features(model);
    // Dizem os papers que fazer sobre o data set completo corre risco de overfitting depois na análise - mas pela minha experiência a eficiência baixa para 20% sem isto
    #[allow(clippy::needless_range_loop)]
    let mut std_and_stat = |vec: &mut Vec<usize>, count| {
        for line in 0..count {
            let row = &mut ds.raw.as_mut().unwrap()[vec[line]];
            for col in &model.selected_features {
                row[*col] = (row[*col] - ds.basic_stats[*col].mean) / ds.basic_stats[*col].sigma;
                // ds.standardization_stats[*col].next(row[*col]);
            }
        }
    };
    for class in 0..NR_CLASSES {
        // Standardizamos as dados de train
        std_and_stat(&mut ds.train_class_idxs[class], model.train_class_count[class]);
        std_and_stat(&mut ds.test_class_idxs[class], model.test_class_count[class]);
    }
}

#[inline]
fn oversample_if_needed_and_correlate(model: &mut Model, ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>) -> usize {
    let mut raw_len = ds.raw.as_ref().unwrap().len();
    let ds_max_count = model.train_class_count.iter().fold(usize::MIN, |max, x| max.max(*x));

    // candidato a paralelização lançando threads em paralelo para cada class
    let mut ds_class_count: usize;
    let mut nr_novos_elementos: usize;

    // o caso principal, para criar dados random diferentes em cada run
    let mut rng = rand::thread_rng();
    let mut between: Uniform<usize>;
    let mut row: [f64; MAX_FEATURES];
    for class in 0..NR_CLASSES {
        ds_class_count = model.train_class_count[class];
        // se nr_elementos > 0, classe desbalanceada - bora balancear com oversample dos dados existentes.
        // Com o tempo iremos balanceando o data set até este ciclo deixar de ser preciso
        nr_novos_elementos = ds_max_count - ds_class_count;
        // utilizamos uma distribuição uniforme para ir buscar indices possiveis aos dados
        between = Uniform::from(0..ds_class_count);

        for _i in 0..nr_novos_elementos {
            // com base no nr aleatorio dos ids de treino da classe, vamos á classe buscar o idx, para ir buscar a row ao ds.train
            row = ds.raw.as_ref().unwrap()[ds.all_class_idxs[class][between.sample(&mut rng)]];
            ds.raw.as_mut().unwrap().push(row);
            // atualizamos a média e a variança
            // e fazemos o trabalho incremental de calculo da mean e var (desvio padrão ** 2)
            ds.correlate_row(&row, raw_len);
            // e aproveitamos mais uma vez a boleia para registar os indices dos novos elementos para ajudar o fit mais á frente
            ds.all_class_idxs[class].push(raw_len);
            raw_len += 1;
        }
        // atualizamos o nr de elementos da class respetiva
        model.train_class_count[class] += nr_novos_elementos;
    }
    // este refere-se ao loop anterior
    // vamos criar os ds de train e test
    let mut train_class_count: usize;
    let mut test_class_count: usize;
    let insert_idx = |class: usize, all_idxs: &[Vec<usize>], vec: &mut Vec<usize>, from: usize, to: usize| {
        let mut idx: usize;
        for i in from..to {
            idx = all_idxs[class][i];
            vec.push(idx);
        }
    };
    for class in 0..NR_CLASSES {
        // vamos buscar 0,7 de cada classe - train - para assegurar que o data set permanece balanceado
        train_class_count = (model.train_class_count[class] as f64 * 0.7).round() as usize;
        // vamos buscar 0,3 de cada classe -  test - para assegurar que o data set permanece balanceado
        test_class_count = (model.train_class_count[class] as f64 * 0.3).round() as usize;

        model.train_class_count[class] = train_class_count;
        model.test_class_count[class] = test_class_count;

        // temos que fazer um shuffle porque como criamos rows com oversampled, vamos tentar que a mistura dos dados reais com os artificiais minimize efeitos "negativos"
        // (whatever this means estatisticamente  :-) mas o raciocinio parece fazer sentido para que no teste exista a probabilidade de existirem dados reais)
        ds.all_class_idxs[class][0..ds_max_count].shuffle(&mut thread_rng());

        insert_idx(class, &ds.all_class_idxs, &mut ds.train_class_idxs[class], 0, train_class_count);
        insert_idx(class, &ds.all_class_idxs, &mut ds.test_class_idxs[class], train_class_count, ds_max_count);
    }
    raw_len
}

#[inline]
pub fn model_fit(ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>, model: &mut Model) {
    // para cada vetor de X de cada classe
    // calcular a var e o sigma de cada coluna (feature) das selected features
    let count_all_train_classes: usize = model.train_class_count.iter().sum();
    let nr_features = model.selected_features.len();
    let mut idx: usize;
    let mut row: &mut DSRow<MAX_FEATURES>;
    let mut col_idx: usize;

    // aqui calculamos os dados necessários para se poder fazer o fit
    for class in 0..NR_CLASSES {
        // atualizamos as probabilidades preliminares de cada elemento em cada classe
        model.class_prior[class] = model.train_class_count[class] as f64 / count_all_train_classes as f64;
        // para cada elemento da classe
        for line in 0..ds.train_class_idxs[class].len() {
            idx = ds.train_class_idxs[class][line]; // vamos buscar o indice da linha - pointer para a linha no train data
            row = &mut ds.raw.as_mut().unwrap()[idx]; // vamos buscar a linha dos dados

            // para cada uma das features selecciondas vamos fazer o fit
            for col in 0..nr_features {
                col_idx = model.selected_features[col];
                model.fit_stats[class][col_idx].next(row[col_idx]);
            }
        }
    }

    // a ideia é calcular a probabilidade de cada classe em cada linha (log likelyhood)
    // vamos buscar as linhas de cada classe
    // ou seja é aqui que é feito o fit
    for class in 0..NR_CLASSES {
        // Para cada linha da classe
        for line in 0..ds.train_class_idxs[class].len() {
            idx = ds.train_class_idxs[class][line]; // vamos buscar o indice da linha - pointer para a linha no train data
            row = &mut ds.raw.as_mut().unwrap()[idx]; // vamos buscar a linha dos dados
                                                      // temos o vetor com o nr de linhas de train, com as probabilidades de cada classe em cada linha
            ds.joint_log_likelihood.push(log_likelihood(model, row));
        }
    }

    // e vamos fazer a previsão para os dados de teste
    // O predict trabalha sobre o fit_stats, que têm que ser calculado antes
    let mut real_class: usize;
    let mut pred: NBGaussianPrediction;
    for class in 0..NR_CLASSES {
        for line in 0..model.test_class_count[class] {
            idx = ds.test_class_idxs[class][line]; // vamos buscar o indice da linha - pointer para a linha no test data
            row = &mut ds.raw.as_mut().unwrap()[idx];
            pred = predict(model, row);
            row[Metric::RainClassForecast as usize] = pred.index as f64;
            real_class = row[Metric::RainClass as usize].round() as usize;
            model.cm[real_class][pred.index] += 1;
        }
    }
    // construimos a confusion matrix
    build_confusion_matrix(model);
    // avaliamos o modelo
    evaluate_model(model);
}

#[inline]
fn build_confusion_matrix(model: &mut Model) {
    // percorremos as linhas
    for class_row in 0..NR_CLASSES {
        model.cm_report[class_row].t_p += model.cm[class_row][class_row]; // percorre a diagonal

        for class_col in 0..NR_CLASSES {
            if class_row != class_col {
                // percorremos a linha da classe
                model.cm_report[class_row].f_n += model.cm[class_row][class_col];
                //percorremos a coluna da classe
                model.cm_report[class_row].f_p += model.cm[class_col][class_row];
                // percorremos todas as linhas e colunas excepto a própria linha e a própria coluna
                model.cm_report[class_col].t_n += model.cm[class_row][class_col];
            }
        }
    }
}

#[inline]
pub fn evaluate_model(model: &mut Model) {
    let mut sum_precision = 0.;
    let mut aux_precision;
    let mut aux_weighted_precision = 0.;

    let mut sum_recall = 0.;
    let mut aux_recall;
    let mut aux_weighted_recall = 0.;

    let mut sum_accuracy = 0.;
    let mut aux_accuracy;
    let mut aux_weighted_accuracy = 0.;

    let mut sum_f1 = 0.;
    let mut aux_f1;
    let mut aux_weighted_f1 = 0.;

    let mut tp_tn;

    let test_total_class_count: f64 = model.test_class_count.iter().sum::<usize>() as f64;
    // isto funciona em articulação com o enum MdlEvalRows, que by desgin respeita os ids das classes
    for class in 0..NR_CLASSES {
        aux_precision = model.cm_report[class].t_p as f64 / (model.cm_report[class].f_p as f64 + model.cm_report[class].t_p as f64);
        model.evaluation[class][MdlEvalCols::Precision as usize] = aux_precision;
        sum_precision += aux_precision;
        aux_weighted_precision += aux_precision * (model.test_class_count[class] as f64 / test_total_class_count);

        aux_recall = model.cm_report[class].t_p as f64 / (model.cm_report[class].f_n as f64 + model.cm_report[class].t_p as f64);
        model.evaluation[class][MdlEvalCols::Recall as usize] = aux_recall;
        sum_recall += aux_recall;
        aux_weighted_recall += aux_recall * (model.test_class_count[class] as f64 / test_total_class_count);

        tp_tn = model.cm_report[class].t_p as f64 + model.cm_report[class].t_n as f64;

        aux_accuracy = tp_tn / (tp_tn + model.cm_report[class].f_p as f64 + model.cm_report[class].f_n as f64);
        model.evaluation[class][MdlEvalCols::Accuracy as usize] = aux_accuracy;
        sum_accuracy += aux_accuracy;
        aux_weighted_accuracy += aux_accuracy * (model.test_class_count[class] as f64 / test_total_class_count);

        aux_f1 = 2. * aux_precision * aux_recall / (aux_precision + aux_recall);
        model.evaluation[class][MdlEvalCols::F1Score as usize] = aux_f1;
        sum_f1 += aux_f1;
        aux_weighted_f1 += aux_f1 * (model.test_class_count[class] as f64 / test_total_class_count);
    }
    model.evaluation[MdlEvalRows::MacroAverage as usize][MdlEvalCols::Precision as usize] = sum_precision / NR_CLASSES as f64;
    model.evaluation[MdlEvalRows::WeightedAverage as usize][MdlEvalCols::Precision as usize] = aux_weighted_precision;

    model.evaluation[MdlEvalRows::MacroAverage as usize][MdlEvalCols::Recall as usize] = sum_recall / NR_CLASSES as f64;
    model.evaluation[MdlEvalRows::WeightedAverage as usize][MdlEvalCols::Recall as usize] = aux_weighted_recall;

    model.evaluation[MdlEvalRows::MacroAverage as usize][MdlEvalCols::Accuracy as usize] = sum_accuracy / NR_CLASSES as f64;
    model.evaluation[MdlEvalRows::WeightedAverage as usize][MdlEvalCols::Accuracy as usize] = aux_weighted_accuracy;

    model.evaluation[MdlEvalRows::MacroAverage as usize][MdlEvalCols::F1Score as usize] = sum_f1 / NR_CLASSES as f64;
    model.evaluation[MdlEvalRows::WeightedAverage as usize][MdlEvalCols::F1Score as usize] = aux_weighted_f1;
}

// e quando se partem as coisas assim, fica óbvio que o Model deve ser mais lean, porque não precisa de andar com os dados todos atrás
// #[allow(dead_code)]
// é para ser usada manualmente para testar e gerar- modelos.
#[inline]
pub fn build_model_and_save_model(db: &Persist, old_curr_model: u32, end_time: CtrlTime, new_curr_model: u32) {
    let mut ds = DataSet::new();
    let mut model = Model::new();

    // vamos buscar os dias todos á bd e aproveitar para determinar no que vem da bd, as contagens das classes e os indices já conhecidos de cada classe
    db.get_ml_raw_data(&mut ds, &mut model).unwrap();

    // aqui vamos fazer o resampling para equilibrar o numero de registos de cada class no dataset, para balancear o algoritmo naive bayes
    // avaliar se há alguma passagem por todos os registos, para calcular já algumas das estatisticas necessárias para a standardização e para o fit
    // e temos que standardizar todas as colunas, para depois tratar do resto
    // vamos depois analisar a correlação entre todas as variaveis - todas as combinações 2 a 2 - para identificar aquelas que estão corr > 0.95
    // e escolher as features para a análise dinamicamente
    prep_data(&mut ds, &mut model);

    // aqui faz o fit - em tese já fizémos as passagens todas lá atrás...e esta será a ultima
    model_fit(&mut ds, &mut model);

    save_model(&model, db, old_curr_model, end_time, new_curr_model)
}

// É só para utilizar qd estou a treinar o modelo
// #[allow(dead_code)]
#[inline]
pub fn choose_best_model(db: &Persist, run_nr: u32) {
    let mut best_model: u32 = 0;
    let mut current_result: f64 = 0.;
    let mut new_result: f64;
    let mut prev_best: u32 = best_model;
    let mut thousands = 0;

    _ = db.clean_explored_models();

    for (pace, (model_id, _i)) in (0..run_nr).enumerate().enumerate() {
        let mut ds = DataSet::new();
        let mut model = Model::new();

        db.get_ml_raw_data(&mut ds, &mut model).unwrap();
        prep_data(&mut ds, &mut model);
        model_fit(&mut ds, &mut model);

        new_result = objetive_function(&model);
        if new_result > current_result {
            prev_best = best_model;
            best_model = model_id as u32;
            current_result = new_result;
        }
        let model_str = data_to_str(&model).unwrap();
        if prev_best != best_model {
            _ = db.update_explored_model(prev_best, false);
        }
        _ = db.insert_explored_model(model_id as u32, model_id as u32 == best_model, model_str);

        if pace % 1000 == 0 {
            thousands += 1;
            print!("{}", thousands);
        }
    }
}

// é utilizada pela função choose best model
// #[allow(dead_code)]
// quero maximizar o melhor F1Score - mas qd mudar de ideias, é mudar a função objetivo
#[inline]
fn objetive_function(model: &Model) -> f64 {
    // let mut objective = 0.;
    // for class in 0..NR_CLASSES {
    //     objective += model.evaluation[class][MdlEvalCols::F1Score as usize];
    // }
    // objective / NR_CLASSES as f64
    model.evaluation[MdlEvalRows::MacroAverage as usize][MdlEvalCols::F1Score as usize]
}

#[inline]
pub fn save_model(model: &Model, db: &Persist, old_curr_model: u32, end_time: CtrlTime, new_curr_model: u32) {
    let model_str = data_to_str(&model).unwrap();
    match db.save_model(old_curr_model, end_time, new_curr_model, model_str) {
        Ok(_) => (),
        Err(err) => error!("Erro a gravar o modelo ml na base de dados.{}", err.to_string()),
    }
}
