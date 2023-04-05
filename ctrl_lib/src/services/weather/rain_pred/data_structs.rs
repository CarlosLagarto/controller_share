use arrayvec::ArrayVec;
use rustc_hash::FxHashMap;

use crate::data_structs::sensor::{daily_value::SensorValue, stat_metric::*};

// este modelo está especializado para o meu problema.  Por exemplo, não foi feito nenhum esforço para generalizar o tratamento das classes
// por definição do modelo são 5 (NR_CLASSES), e isso está espelhado e hardcoded no código por questões de eficiência
/// Dimension 48
pub const RAIN_CLASS_KEYS: [u8; 47] = [
    Metric::AvgHumidity as u8,
    Metric::AvgPressure as u8,
    Metric::AvgWindSpeed as u8,
    Metric::MaxHumidity as u8,
    Metric::MaxTemp as u8,
    Metric::MinHumidity as u8,
    Metric::MinTemp as u8,
    Metric::SumRain as u8,
    Metric::AvgTemp as u8,
    Metric::MaxPressure as u8,
    Metric::MinPressure as u8,
    Metric::MaxWindSpeed as u8,
    Metric::MinWindSpeed as u8,
    Metric::SolarRadiation as u8,
    Metric::AvgWindDirection as u8,
    Metric::TempAt0 as u8,
    Metric::TempAt6 as u8,
    Metric::TempAt12 as u8,
    Metric::TempAt18 as u8,
    Metric::PressAt0 as u8,
    Metric::PressAt6 as u8,
    Metric::PressAt12 as u8,
    Metric::PressAt18 as u8,
    Metric::HrAt0 as u8,
    Metric::HrAt6 as u8,
    Metric::HrAt12 as u8,
    Metric::HrAt18 as u8,
    Metric::WsAt0 as u8,
    Metric::WsAt6 as u8,
    Metric::WsAt12 as u8,
    Metric::WsAt18 as u8,
    Metric::WdAt0 as u8,
    Metric::WdAt6 as u8,
    Metric::WdAt12 as u8,
    Metric::WdAt18 as u8,
    Metric::AvgDwp as u8,
    Metric::MaxDwp as u8,
    Metric::MinDwp as u8,
    Metric::DwpAt0 as u8,
    Metric::DwpAt6 as u8,
    Metric::DwpAt12 as u8,
    Metric::DwpAt18 as u8,
    Metric::PressureDwpRatio as u8,
    Metric::HumidityGtERatio as u8,
    Metric::RainClass as u8,
    Metric::RainClassForecast as u8,
    Metric::DayNr as u8,
];

/// Dimension 360
pub const STANDARDIZE_KEYS: [usize; 45] = [
    Metric::AvgHumidity as usize,
    Metric::AvgPressure as usize,
    Metric::AvgWindSpeed as usize,
    Metric::MaxHumidity as usize,
    Metric::MaxTemp as usize,
    Metric::MinHumidity as usize,
    Metric::MinTemp as usize,
    Metric::SumRain as usize,
    Metric::AvgTemp as usize,
    Metric::MaxPressure as usize,
    Metric::MinPressure as usize,
    Metric::MaxWindSpeed as usize,
    Metric::MinWindSpeed as usize,
    Metric::SolarRadiation as usize,
    Metric::AvgWindDirection as usize,
    Metric::TempAt0 as usize,
    Metric::TempAt6 as usize,
    Metric::TempAt12 as usize,
    Metric::TempAt18 as usize,
    Metric::PressAt0 as usize,
    Metric::PressAt6 as usize,
    Metric::PressAt12 as usize,
    Metric::PressAt18 as usize,
    Metric::HrAt0 as usize,
    Metric::HrAt6 as usize,
    Metric::HrAt12 as usize,
    Metric::HrAt18 as usize,
    Metric::WsAt0 as usize,
    Metric::WsAt6 as usize,
    Metric::WsAt12 as usize,
    Metric::WsAt18 as usize,
    Metric::WdAt0 as usize,
    Metric::WdAt6 as usize,
    Metric::WdAt12 as usize,
    Metric::WdAt18 as usize,
    Metric::AvgDwp as usize,
    Metric::MaxDwp as usize,
    Metric::MinDwp as usize,
    Metric::DwpAt0 as usize,
    Metric::DwpAt6 as usize,
    Metric::DwpAt12 as usize,
    Metric::DwpAt18 as usize,
    Metric::PressureDwpRatio as usize,
    Metric::HumidityGtERatio as usize,
    Metric::DayNr as usize,
];

pub const MAX_EVALUATION_COLS: usize = 4;
pub const MAX_EVALUATION_LINES: usize = 7;

/// Dimension 392 para MAX_FEATURES
pub type DSRow<const COLS: usize> = [f64; COLS];
pub type DSIdx = FxHashMap<u8, usize>;
/// Dimension 224
pub type ModelEvaluation = [[f64; MAX_EVALUATION_COLS]; MAX_EVALUATION_LINES];

// Isto é utilizado no WeatherInner, porque a ler da BD, não temos a certeza de ter todas as colunas por questões de falha nos sensores,
// mas ainda assim queremos continuar e obter os dados que existem - as decisões de go/no go para os calculos são posteriores, por há alguns
// cálculos que podem ser feitos com defaults
/// Dimension 824 para uma capacidade de MAX_FEATURES
pub struct Vector<const CAP: usize> {
    // estes buffers é para reclamar a memória no inicio do programa e reutiliza-la durante a execução, evitando assim alguns mallocs durante a execução
    pub data: ArrayVec<SensorValue, CAP>,
    pub idx: DSIdx,
}

impl<const CAP: usize> Vector<{ CAP }> {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            data: ArrayVec::<SensorValue, { CAP }>::new(),
            idx: FxHashMap::default(),
        }
    }
    #[inline]
    pub fn push(&mut self, data: SensorValue) -> Option<usize> {
        let key = data.id;
        self.data.push(data);
        self.idx.insert(key, self.data.len() - 1)
    }

    #[inline]
    pub fn remove(&mut self, metric: Metric) -> Option<SensorValue> {
        let u_metric = metric as u8;
        if let Some(idx) = self.idx.get(&u_metric) {
            let mut u_idx = *idx;
            self.idx.remove(&u_metric);

            // a questão aqui é que removendo o elemento indice idx do array,  o hashmap fica com todos os indices > idx marados, e este é o acerto.
            // Pouco eficiente, mas isto é só para testes e simulaçao, por causa das colisões com dados anteriores - porque não é chamado de mais lado nenhum...
            for (_key, val) in self.idx.iter_mut() {
                if val > &mut u_idx {
                    *val -= 1;
                }
            }
            Some(self.data.remove(u_idx))
        } else {
            None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.idx.clear();
        self.data.clear();
    }
}

/// Dimension 8
/// Model Confusion Matrix data
#[derive(Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CMData {
    pub t_p: u16,
    pub t_n: u16,
    pub f_p: u16,
    pub f_n: u16,
}

/// Dimension 10304
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub fit_stats: [ArrayVec<WelfordMeanAndVar, MAX_FEATURES>; NR_CLASSES],
    pub selected_features: Vec<usize>,

    pub class_prior: [f64; NR_CLASSES],

    pub cm: [[u16; NR_CLASSES]; NR_CLASSES],
    pub cm_report: [CMData; NR_CLASSES],
    pub evaluation: ModelEvaluation,

    pub train_class_count: [usize; NR_CLASSES],
    pub test_class_count: [usize; NR_CLASSES],
}

impl Model {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        #[rustfmt::skip]
        let mut fit_stats = [  ArrayVec::<WelfordMeanAndVar, MAX_FEATURES>::new(),
                                                                    ArrayVec::<WelfordMeanAndVar, MAX_FEATURES>::new(),
                                                                    ArrayVec::<WelfordMeanAndVar, MAX_FEATURES>::new(),
                                                                    ArrayVec::<WelfordMeanAndVar, MAX_FEATURES>::new(),
                                                                    ArrayVec::<WelfordMeanAndVar, MAX_FEATURES>::new() ];
        for vec in fit_stats.iter_mut().take(NR_CLASSES) {
            for _feature in 0..MAX_FEATURES {
                vec.push(WelfordMeanAndVar::default());
            }
        }

        Self {
            fit_stats,
            test_class_count: [0; NR_CLASSES],
            train_class_count: [0; NR_CLASSES],
            selected_features: Vec::with_capacity(MAX_FEATURES),

            cm: [[0; NR_CLASSES]; NR_CLASSES],
            cm_report: [CMData::default(); NR_CLASSES],
            evaluation: [[0.; MAX_EVALUATION_COLS]; MAX_EVALUATION_LINES],
            class_prior: [0.; NR_CLASSES],
        }
    }
}

/// Dimension 40
// pub const VAR_SMOOTHING: f64 = 0.000000001;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WelfordMeanAndVar {
    pub mean: f64,
    pub k: usize,
    pub s: f64,
    pub max: f64,
    pub min: f64,
}

impl Default for WelfordMeanAndVar {
    #[rustfmt::skip]
    #[inline]
    fn default() -> Self { Self { mean: 0., k: 0, s: 0., max: f64::MIN, min: f64::MAX, } }
}

impl WelfordMeanAndVar {
    #[inline]
    pub fn next(&mut self, x: f64) {
        self.k += 1;
        let mnext = self.mean + (x - self.mean) / self.k as f64;
        self.s += (x - self.mean) * (x - mnext);
        self.mean = mnext;

        self.max = self.max.max(x);
        self.min = self.min.min(x);
    }

    #[inline]
    #[rustfmt::skip]
    pub fn var(&self) -> f64 {
        let mut res: f64 = 0.;
        if self.k > 1 { res = self.s / self.k as f64; }
        res
    }

    #[inline]
    #[rustfmt::skip]
    pub fn var_sample(&self) -> f64 {
        let mut res: f64 = 0.;
        if self.k > 1 { res = self.s / (self.k as f64 - 1.); }
        res
    }
}

#[derive(Debug)]
/// Dimension 128
pub struct NBGaussianPrediction {
    /// O indice nos array jll, log e prob que é sinónimo da classe prevista
    pub index: usize,
    /// O vetor com a jll - Joint LikeLyhood do vetor com os dados passados para a previsão
    pub jll: [f64; NR_CLASSES],
    /// O vetor com os logaritmos da probabilidade
    pub log_probability: [f64; NR_CLASSES],
    /// O vetor com a probabilidade
    pub probability: [f64; NR_CLASSES],
}

impl Default for NBGaussianPrediction {
    #[inline]
    fn default() -> Self {
        Self {
            index: 0,
            jll: [0.; NR_CLASSES],
            log_probability: [0.; NR_CLASSES],
            probability: [0.; NR_CLASSES],
        }
    }
}

impl NBGaussianPrediction {
    #[rustfmt::skip]
    #[inline]
    pub fn get_max_jll(&self) -> f64 { self.jll[self.index] }

    pub fn rain_probability(&self) -> f32{
        let mut rain_probability = 0.;
        for i in 1..NR_CLASSES{
            rain_probability += self.probability[i];
        }
        rain_probability as f32
    }
}

#[inline]
pub fn max_index(vec: &[f64]) -> usize {
    let mut class_idx: usize = 0;
    let mut max = f64::MIN;
    for (idx, item) in vec.iter().enumerate() {
        if item > &max {
            max = *item;
            class_idx = idx;
        }
    }
    class_idx
}

#[inline]
pub fn rain_class_from_rain_mm(sum_rain: f64) -> f64 {
    match sum_rain {
        x if x <= 0. => 0.,
        x if x <= 5. => 1.,
        x if x <= 10. => 2.,
        x if x <= 20. => 3.,
        x if x > 20. => 4.,
        _ => 0.,
    }
}
