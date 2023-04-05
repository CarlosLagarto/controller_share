use std::time::Instant;

use ctrl_lib::{
    app_time::ctrl_time::*,
    data_structs::sensor::stat_metric::*,
    db::db_sql_lite::*,
    lib_serde::data_from_str,
    services::weather::{rain_pred::data_structs::*, sources::tempest::data_structs::Tempest},
    utils::*,
};

use crate::integration::naive_bayes::{
    code::{data_munch::*, data_structs::*, db_model::DBModelWeatherNBMunch},
    tests_aux::*,
};

#[test]
fn test_correlation() {
    let x = [1., 2., 4., 5., 6., 7., 8., 8., 10., 12.];
    let y = [3., 4., 5., 6., 7., 8., 9., 9., 10., 11.];
    let mut w = PearsonCorrelation::new();

    for i in 0..x.len() {
        w.next(x[i], y[i], i as f64 + 1.);
    }
    assert!((w.corr() - 0.9909189986511214).abs() < 0.00001);
}

#[test]
fn test_get_ml_raw_data() {
    let mut ds = DataSet::<MAX_DAILY_ROWS, MAX_FEATURES>::new();
    let mut model = Model::new();
    let db = Persist::new();
    let t = Instant::now();
    let res = db.get_ml_raw_data(&mut ds, &mut model);
    let dur = (Instant::now() - t).as_nanos() as u64;
    println!("get_ml_raw_data: {}", elapsed_dyn(dur));
    // dry run - smoke test
    assert!(res.is_ok());
}

#[test]
fn test_prep_data() {
    let mut ds = DataSet::new();
    let mut model = Model::new();
    let db = Persist::new();
    let mut t = Instant::now();
    let res = db.get_ml_raw_data(&mut ds, &mut model);
    println!("get_ml_raw_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    prep_data_python(&mut ds, &mut model);
    println!("prep_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));
    // dry run - smoke test
    assert!(res.is_ok());
}

#[test]
fn test_model_fit() {
    let mut ds = DataSet::new();
    let mut model = Model::new();
    let db = Persist::new();
    let mut t = Instant::now();
    let res = db.get_ml_raw_data(&mut ds, &mut model);
    println!("get_ml_raw_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    prep_data_python(&mut ds, &mut model);
    println!("prep_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    model_fit(&mut ds, &mut model);
    println!("model_fit: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    // dry run - smoke test
    assert!(res.is_ok());
}

const NR_AUX_IDXS: usize = 367;

#[inline]
fn prep_data_python(ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>, model: &mut Model) {
    oversample_if_needed_and_correlate_for_testing(model, ds);
    standardize(ds, model);
}

// esta deverá ser igual á de produção, com unica a diferença de que os indices aleatorios são obtidos hardcoded, para poder comparar com a implementação python
fn oversample_if_needed_and_correlate_for_testing(model: &mut Model, ds: &mut DataSet<MAX_DAILY_ROWS, MAX_FEATURES>) -> usize {
    let mut raw_len = ds.raw.as_ref().unwrap().len();
    let ds_max_count = model.train_class_count.iter().fold(usize::MIN, |max, x| max.max(*x));
    let random_seq: [Vec<usize>; NR_AUX_IDXS] = RandomTestIdxs::new().data;

    // candidato a paralelização lançando threads em paralelo para cada class
    let mut ds_class_count: usize;
    let mut nr_novos_elementos: usize;
    let mut new_recs_nr: usize = 0;

    let mut rnd_seq: &Vec<usize>;
    let mut row: [f64; MAX_FEATURES];
    for class in 0..NR_CLASSES {
        ds_class_count = model.train_class_count[class];
        nr_novos_elementos = ds_max_count - ds_class_count;
        rnd_seq = &random_seq[i64::max(ds_class_count as i64 - 1, 0) as usize];
        for i in 0..nr_novos_elementos {
            // com base no nr aleatorio dos ids de treino da classe, vamos á classe buscar o idx, para ir buscar a row aos ids do ds.raw
            row = ds.raw.as_ref().unwrap()[ds.all_class_idxs[class][rnd_seq[i % ds_class_count]]];
            ds.raw.as_mut().unwrap().push(row);
            ds.all_class_idxs[class].push(raw_len);
            // atualizamos a média e a variança
            // e fazemos o trabalho incremental de calculo da mean e var (desvio padrão ** 2)
            ds.correlate_row(&row, raw_len);
            raw_len += 1;
        }
        // atualizamos o nr de elementos da class respetiva
        model.train_class_count[class] += nr_novos_elementos;
        new_recs_nr += nr_novos_elementos;
    }
    // este refere-se ao loop anterior
    println!("Passagem 3.5 por {} linhas - para balancear os dados", new_recs_nr);
    println!(
        "Passagem 4 por {} linhas - para separar train e test, e simultaneamento criar os dados para standardizar e correlacionar",
        ds.raw.as_ref().unwrap().len()
    );
    // vamos criar os ds de train e test
    let mut train_class_count: usize;
    let mut test_class_count: usize;
    let insert_idx = |class: usize, vec: &mut Vec<usize>, from: usize, to: usize, rnd_seq: &Vec<usize>| {
        let mut idx: usize;
        for val in rnd_seq.iter().take(to).skip(from) {
            idx = ds.all_class_idxs[class][*val];
            vec.push(idx);
        }
    };

    for class in 0..NR_CLASSES {
        // vamos buscar 0,7 de cada classe - train - para assegurar que o data set permanece balanceado
        train_class_count = (model.train_class_count[class] as f64 * 0.7).round() as usize;
        test_class_count = (model.train_class_count[class] as f64 * 0.3).round() as usize;
        model.train_class_count[class] = train_class_count;
        model.test_class_count[class] = test_class_count;
        // temos que fazer um shuffle porque como criamos rows com oversampled, vamos tentar que a mistura dos dados reais com os artificiais minimize efeitos "negativos"
        // (whatever this means estatisticamente  :-) mas o raciocinio parece fazer sentido)
        // e vamos buscar a ultima linha dos dados auxiliares de testes para compararmos os dados com a implementação python
        rnd_seq = &random_seq[366];
        insert_idx(class, &mut ds.train_class_idxs[class], 0, train_class_count, rnd_seq);
        // vamos buscar 0,3 de cada classe -  test - para assegurar que o data set permanece balanceado
        insert_idx(class, &mut ds.test_class_idxs[class], train_class_count, ds_max_count, rnd_seq);
    }
    raw_len
}

#[test]
fn test_save_model() {
    let mut ds = DataSet::new();
    let mut model = Model::new();
    let db = &Persist::new();
    let mut t = Instant::now();
    let res = db.get_ml_raw_data(&mut ds, &mut model);
    println!("get_ml_raw_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    prep_data_python(&mut ds, &mut model);
    println!("prep_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    #[rustfmt::skip]
    let mut python_features: [usize; 34] = [
        Metric::DayNr as usize, Metric::AvgTemp as usize, Metric::MaxTemp as usize, Metric::MinTemp as usize, Metric::AvgPressure as usize,
        Metric::AvgHumidity as usize, Metric::MaxHumidity as usize, Metric::MinHumidity as usize, Metric::SumRain as usize, Metric::AvgWindSpeed as usize,
        Metric::MaxWindSpeed as usize, Metric::MinWindSpeed as usize, Metric::SolarRadiation as usize, Metric::AvgWindDirection as usize,
        Metric::TempAt0 as usize, Metric::TempAt6 as usize, Metric::TempAt18 as usize, Metric::PressAt0 as usize, Metric::HrAt0 as usize,
        Metric::HrAt6 as usize, Metric::HrAt12 as usize, Metric::HrAt18 as usize, Metric::WsAt0 as usize, Metric::WsAt6 as usize, Metric::WsAt12 as usize,
        Metric::WsAt18 as usize, Metric::WdAt0 as usize, Metric::WdAt6 as usize, Metric::WdAt12 as usize, Metric::WdAt18 as usize, Metric::AvgDwp as usize,
        Metric::DwpAt0 as usize, Metric::PressureDwpRatio as usize, Metric::HumidityGtERatio as usize,
    ];

    python_features.sort();
    let mut all_equal = true;
    for i in 0..python_features.len() {
        all_equal = all_equal && (python_features[i] == model.selected_features[i]);
    }
    assert!(all_equal, "todas as features são iguais - fixe");

    t = Instant::now();
    model_fit(&mut ds, &mut model);
    println!("model_fit: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    save_model(&model, db, 0, CtrlTime::sys_time(), 1);
    println!("save_model: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    pretty_print_cm(&model);
    // dry run - smoke test
    assert!(res.is_ok());
}

#[test]
fn test_save_model_no_python() {
    let mut ds = DataSet::new();
    let mut model = Model::new();
    let db = &Persist::new();
    let mut t = Instant::now();
    let res = db.get_ml_raw_data(&mut ds, &mut model);
    println!("get_ml_raw_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    prep_data(&mut ds, &mut model);
    println!("prep_data: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    model_fit(&mut ds, &mut model);
    println!("model_fit: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    t = Instant::now();
    save_model(&model, db, 0, CtrlTime::sys_time(), 1);
    println!("save_model: {}", elapsed_dyn((Instant::now() - t).as_nanos() as u64));

    pretty_print_cm(&model);
    // dry run - smoke test
    assert!(res.is_ok());
}

// isto deixou de ser preciso porque mudei a estratégia de testes, mas deixo aqui just in case para o futuro
// #[rustfmt::skip]
//     const SIGMA : [[f64;34];5] = [[7.00100025e-01, 1.81065400e+00, 1.25932207e+00, 1.64044784e+00, 6.70485677e-01, 8.90888827e-01, 1.27227485e+00, 6.05163751e-01, 1.41699825e-09,
//                                    2.37728766e-01, 3.45799785e-01, 3.51736275e-01, 1.17838855e+00, 3.43654659e-01, 1.78145423e+00, 1.71858194e+00, 1.66487316e+00, 6.92503894e+00,
//                                    1.27515335e+00, 1.20045954e+00, 6.78685193e-01, 8.74964402e-01, 1.78570408e-01, 2.55345401e-01, 6.81352879e-01, 1.58530219e+00, 1.56296471e+00,
//                                    1.35393157e+00, 1.69004810e+00, 1.83145307e+00, 1.90608442e+00, 1.78967900e+00,  2.33706151e+00, 1.05301460e+00],
//                                   [8.45580074e-01, 1.09998717e+00, 8.37622038e-01, 1.26559083e+00, 6.56644429e-01, 4.95402875e-01, 4.17793840e-01, 5.91130177e-01, 1.00432995e-02,
//                                    3.19422763e-01, 3.30515372e-01, 5.78931906e-01, 9.62262936e-01, 5.97371407e-01, 1.04448712e+00, 1.26913260e+00, 9.86199729e-01, 2.04807987e-02,
//                                    4.73908016e-01, 4.88669548e-01, 7.67396315e-01, 6.32839787e-01, 3.32218621e-01, 3.52595616e-01, 5.63387934e-01, 6.30157819e-01, 1.23922859e+00,
//                                    1.38391412e+00, 1.03141162e+00, 8.51143369e-01, 1.20921689e+00, 1.20694103e+00, 2.97144881e-01, 1.18459736e+00],
//                                   [1.13751835e+00, 1.06858039e+00, 1.05289140e+00, 1.07981087e+00, 7.96642013e-01, 1.58863299e+00, 2.08625006e+00, 1.30049567e+00, 1.37449034e-02,
//                                    4.89397951e-01, 3.86191087e-01, 9.96334058e-01, 6.44106064e-01, 1.06997624e+00, 1.25825977e+00, 1.01545810e+00, 9.78884105e-01, 2.28398053e-02,
//                                    1.81081320e+00, 1.52058361e+00, 1.12537167e+00, 1.33486654e+00, 4.61972703e-01, 3.51456027e-01, 3.58878170e-01, 8.14081677e-01, 8.64305119e-01,
//                                    7.09506524e-01, 7.98696948e-01, 1.02412979e+00, 5.72779694e-01, 7.52305259e-01, 2.70047002e-02, 1.16149772e+00],
//                                   [5.18490778e-01, 6.84707107e-01, 1.11173990e+00, 4.68335012e-01, 7.55911429e-01, 9.24438377e-01, 6.88962686e-01, 1.10215433e+00, 6.28267797e-02,
//                                    1.44895737e+00, 1.91850395e+00, 1.27568063e+00, 6.98475859e-01, 6.42386778e-01, 5.08002346e-01, 6.32608038e-01, 6.94573439e-01, 3.22852069e-02,
//                                    7.57352590e-01, 1.20968246e+00, 1.02143379e+00, 1.06473701e+00, 2.70921259e+00, 2.65259128e+00, 6.73299325e-01, 6.21746124e-01, 7.01938231e-01,
//                                    5.05321306e-01, 5.69989961e-01, 3.76159298e-01, 3.40661560e-01, 2.55371049e-01, 8.59658616e-03, 4.92820960e-01],
//                                   [1.13133309e+00, 5.01949331e-01, 6.88448356e-01, 7.18888671e-01, 4.43639894e-01, 5.68944572e-01, 4.20994524e-01, 8.35461494e-01, 3.33312394e-01,
//                                    1.46293466e+00, 9.10319902e-01, 1.37769142e+00, 9.59263437e-01, 3.65323449e-01, 6.62322437e-01, 5.37899031e-01, 5.48484467e-01, 3.10589330e-02,
//                                    4.76688764e-01, 3.26213531e-01, 6.21287653e-01, 4.01146482e-01, 4.82408675e-01, 5.39199620e-01, 1.63366134e+00, 1.04336712e+00, 7.63360051e-01,
//                                    3.84044425e-01, 4.25106720e-01, 3.94164513e-01, 8.69773054e-01, 9.60087926e-01, 3.42898452e-02, 8.45176889e-01]];
// #[rustfmt::skip]
//     const THETA : [[f64;34];5] =  [[-0.10841840, -0.07570960,  0.15768154, -0.32975941,  0.61365975, -0.61846472, -0.44696327, -0.61321015, -0.90519962, -0.34492544,
//                                     -0.24777999, -0.36193968,  0.54766997, -0.96120257, -0.25744355, -0.40134211,  0.17178023, -0.12436029, -0.33728786, -0.28050593,
//                                     -0.69152989, -0.71122744, -0.33979751, -0.38585768, -0.30307910,  0.25317832,  0.10395005, -0.47017330,  0.27427166,  0.20122817,
//                                     -0.48670443, -0.41686649,  0.18335304, -0.20719744],
//                                    [ 0.33852586,  0.03997419,  0.05280596,  0.04949171,  0.53005656,  0.19325234,  0.26875651,  0.09648488, -0.83624123, -0.46610447,
//                                     -0.47369983, -0.28591741,  0.11717231, -0.41511653,  0.00406967, -0.05066401,  0.10387923,  0.10108198,  0.23428256,  0.33668412,
//                                      0.08329698, -0.00656624, -0.3447832 , -0.34573857, -0.46500750, -0.07118613, -0.03070556, -0.02770895,  0.47522435,  0.44556311,
//                                      0.18664864,  0.17165268, -0.04166833,  0.17116132],
//                                    [ 0.37863263,  0.10692682,  0.01620382,  0.20559113,  0.18320180, -0.07846414, -0.29523462,  0.04789572, -0.32007657, -0.14579102,
//                                     -0.20067923,  0.1392263 , -0.23863951,  0.21686935,  0.21016286,  0.22784811,  0.04932955,  0.07035352, -0.41594762, -0.25746794,
//                                      0.17906016,  0.08337507,  0.09386805,  0.0314552 , -0.21880774, -0.21466162, -0.01625822,  0.00260445, -0.19269216, -0.33983747,
//                                      0.03166764, -0.13370807,  0.00510457, -0.09056743],
//                                    [-0.70776841, -0.24436419, -0.28685437, -0.07607116, -0.98878692,  0.28611944,  0.23843441,  0.18308955,  0.38126927,  0.70399094,
//                                      0.85675340,  0.29490250, -0.40066966,  0.40721148, -0.02364034,  0.18856510, -0.46507770, -0.14777685,  0.08357917, -0.04371685,
//                                      0.42725784,  0.44309669,  0.89300214,  0.80687517,  0.60149067, -0.23159494,  0.06475864,  0.4820567 , -0.40436233,  0.06160232,
//                                     -0.06891102,  0.02788219, -0.01297966, -0.02536483],
//                                    [ 0.07121773, -0.01151274, -0.05112138, -0.05044609, -0.30481225,  0.14310949,  0.11599415,  0.21293212,  1.64966915,  0.33455669,
//                                      0.16801736,  0.29998679, -0.00629199,  0.82025619, -0.12019241, -0.19565804,  0.01213659,  0.03955565,  0.28977663,  0.18720473,
//                                     -0.03390500,  0.13730495, -0.21818241, -0.06156925,  0.50607295,  0.37319889,  0.07290024, -0.14312234, -0.17695087, -0.43621930,
//                                     0.11447772,  0.08866939,  0.00290845, -0.00516121]];
// #[rustfmt::skip]
//     const PYTHON_FEATURES: [usize; 34] = [Metric::DayNr as usize, Metric::AvgTemp as usize, Metric::MaxTemp as usize, Metric::MinTemp as usize,
//         Metric::AvgPressure as usize, Metric::AvgHumidity as usize, Metric::MaxHumidity as usize, Metric::MinHumidity as usize, Metric::SumRainForForecast as usize,
//         Metric::AvgWindSpeed as usize, Metric::MaxWindSpeed as usize, Metric::MinWindSpeed as usize, Metric::SolarRadiation as usize,
//         Metric::AvgWindDirection as usize, Metric::TempAt0 as usize, Metric::TempAt6 as usize, Metric::TempAt18 as usize, Metric::PressAt0 as usize,
//         Metric::HrAt0 as usize, Metric::HrAt6 as usize, Metric::HrAt12 as usize, Metric::HrAt18 as usize, Metric::WsAt0 as usize, Metric::WsAt6 as usize,
//         Metric::WsAt12 as usize, Metric::WsAt18 as usize, Metric::WdAt0 as usize, Metric::WdAt6 as usize, Metric::WdAt12 as usize, Metric::WdAt18 as usize,
//         Metric::AvgDwp as usize, Metric::DwpAt0 as usize, Metric::PressureDwpRatio as usize, Metric::HumidityGtERatio as usize,
//     ];
// #[rustfmt::skip]
// const X_PYTHON: [f64;34] = [  0.70096445,  1.29317612,  1.26579729,  0.90707758,  0.00530831, -0.06260564,  0.46070133, -0.35921587, -0.90519962, -0.95110721,
//                              -1.04488875, -0.13677816,  1.21620761, -1.00910928,  1.00314541,  0.76964467,  1.64576586,  0.06780824,  0.77638564,  0.86754455,
//                              -0.46671905, -0.80034511, -0.70633439, -0.65751874, -1.16624811, -0.07919513,  1.57135307, -1.09368371,  1.14548531,  0.94420921,
//                               1.22182516,  1.45025643, -0.13843077,  1.08675278];
// fn update_row() -> DSRow<{ MAX_FEATURES }> {
//     let mut x_rust: DSRow<{ MAX_FEATURES }> = [0.; MAX_FEATURES];
//     for (idx, val) in X_PYTHON.iter().enumerate() {
//         x_rust[PYTHON_FEATURES[idx]] = *val;
//     }
//     x_rust
// }

// fn update_fit_mean_model(model: &mut Model) {
//     for (class, class_vec) in SIGMA.iter().enumerate() {
//         for (idx, f) in class_vec.iter().enumerate() {
//             model.fit_stats[class][PYTHON_FEATURES[idx]].s = *f * 124.;
//             model.fit_stats[class][PYTHON_FEATURES[idx]].k = 125;
//             model.fit_stats[class][PYTHON_FEATURES[idx]].mean = THETA[class][idx];
//         }
//     }
// }

// Isto é só para utilizar quando estiver a treinar o modelo
#[test]
#[ignore]
fn test_choose_best_model() {
    let db = Persist::new();
    choose_best_model(&db, 100000);
}

// Isto está aqui porque foi para auxiliar a pensar no problema das permutações e combinações
// #[test]
// fn test_cols_combinations_and_permutations() {
//     use itertools::Itertools;
//     let x = vec![1, 2, 3, 4, 5, 6];
//     let y = vec![1, 2, 3, 4, 5, 6];

//     for v in x.into_iter().permutations(2) {
//         println!("{:?}", v);
//     }
//     println!();
//     println!();
//     for v in y.into_iter().combinations_with_replacement(2) {
//         println!("{:?}", v);
//     }
//     // println!("{:?}", x.into_iter().permutations(2));
// }

#[test]
fn test_precision() {
    let mut model = Model::new();

    for i in 0..NR_CLASSES {
        model.cm_report[i].t_p = 10;
        model.cm_report[i].f_p = 1;
        model.cm_report[i].t_n = 5;
        model.cm_report[i].f_n = 1;
    }
    model.cm_report[0].f_p = 2;

    model.test_class_count[0] = 4;
    model.test_class_count[1] = 2;
    model.test_class_count[2] = 6;
    model.test_class_count[3] = 7;
    model.test_class_count[4] = 30;

    evaluate_model(&mut model);

    assert!(model.evaluation[5][0] - 0.893939 < 0.0001);
    assert!(model.evaluation[6][0] - 0.902907 < 0.0001);
}

#[test]
fn test_top_level_tempest() {
    let buf = r#"
                    {
                        "serial_number": "SK-00008453",
                        "type":"rapid_wind",
                        "hub_sn": "HB-00000001",
                        "ob":[1493322445,2.3,128]
                    }"#;
    let rec: Tempest = data_from_str(&buf).unwrap();
    
    if let Tempest::RapidWind(rapid) = rec {
        // println!("{}", rapid.hub_sn)
        assert!(rapid.hub_sn == "HB-00000001".to_string());
    }else{
        assert!(false);
    }
}
