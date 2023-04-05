use arrayvec::ArrayVec;
use itertools::Itertools;

use ctrl_lib::{data_structs::sensor::stat_metric::*, services::weather::rain_pred::data_structs::*};

// os id das classes tem que aparecer sempre primeiro porque há lógica dependente disso
#[repr(u8)]
pub enum MdlEvalRows {
    ClassRainZero = 0,
    ClassRainLessThan5 = 1,
    ClassRainLessThan10 = 2,
    ClassRainLessThan20 = 3,
    ClassRainMoreThan20 = 4,
    MacroAverage = 5,
    WeightedAverage = 6,
}

#[repr(u8)]
pub enum MdlEvalCols {
    /// Importante para a identificação de dos casos positivos minimizando os falsos positivos <br><br>
    /// Ou seja, interessa para não pararmos a rega por falsos positivos, quando deviamos regar
    Precision = 0,
    /// Sensibilidade ao ratio de True Positivos<br><br>
    /// Indica os falsos negativos (tegamos, quando podiamos não ter regado)
    Recall = 1,
    /// Imagem global da precisão, mas pouco fiável para datasets  não balanceados
    Accuracy = 2,
    /// Média harmónica da Precision e do Recall.  <br><br>
    /// Compromisso entre elevados falsos positivos e falsos negativos
    F1Score = 3,
    // Support = 4,
}

/// Dimension 116872, com os parametros definidos para o array
pub struct DataSet<const ROWS: usize, const COLS: usize> {
    // estes buffers é para reclamar a memória no inicio do programa e reutiliza-la durante a execução, evitando assim alguns mallocs durante a execução
    pub raw: Option<Vec<DSRow<COLS>>>,
    pub all_class_idxs: [Vec<usize>; NR_CLASSES],
    pub cols: usize,

    pub train_class_idxs: [Vec<usize>; NR_CLASSES],
    pub test_class_idxs: [Vec<usize>; NR_CLASSES],

    pub column_pair: Vec<Vec<usize>>,
    pub pearson_correlation: [[PearsonCorrelation; COLS]; COLS], // matriz com todas as correlações entre os pares
    pub basic_stats: ArrayVec<BasicStats, MAX_FEATURES>,

    pub joint_log_likelihood: Vec<[f64; NR_CLASSES]>,
}

impl<const ROWS: usize, const COLS: usize> DataSet<{ ROWS }, { COLS }> {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        let mut ds_sigmas = ArrayVec::<f64, MAX_FEATURES>::new();
        for _feature in 0..MAX_FEATURES {
            ds_sigmas.push(0.);
        }
        let mut basic_stats = ArrayVec::<BasicStats, MAX_FEATURES>::new();
        for _feature in 0..MAX_FEATURES {
            basic_stats.push(BasicStats::default());
        }

        Self {
            raw: Some(Vec::with_capacity(ROWS)),
            all_class_idxs: [
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
            ],
            cols: { COLS },

            train_class_idxs: [
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
            ],
            test_class_idxs: [
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
                Vec::with_capacity(ROWS),
            ],

            column_pair: STANDARDIZE_KEYS.into_iter().combinations_with_replacement(2).collect(),
            pearson_correlation: [[PearsonCorrelation::new(); { COLS }]; { COLS }],
            basic_stats,

            joint_log_likelihood: Vec::with_capacity(MAX_DAILY_ROWS),
        }
    }

    #[inline]
    pub fn push(&mut self, row: [f64; COLS]) {
        self.raw.as_mut().unwrap().push(row);
    }

    #[inline]
    pub fn correlate_row(&mut self, row: &[f64; MAX_FEATURES], row_nr: usize) {
        for pair in &self.column_pair {
            self.pearson_correlation[pair[0]][pair[1]].next(row[pair[0]], row[pair[1]], row_nr as f64 + 1.);
        }
    }

    #[inline]
    pub fn select_features(&mut self, model: &mut Model) {
        let mut columns_to_left_out: Vec<usize> = Vec::with_capacity(MAX_FEATURES);

        for pair in &self.column_pair {
            if pair[0] != pair[1] && self.pearson_correlation[pair[0]][pair[1]].corr().abs() > 0.95 {
                columns_to_left_out.push(pair[1]);
            }
        }
        columns_to_left_out.sort();
        for col in STANDARDIZE_KEYS {
            if !columns_to_left_out.contains(&col) {
                model.selected_features.push(col);
            }
        }
    }
}

/// Dimension 24
#[derive(Default)]
pub struct BasicStats {
    pub mean: f64,
    pub var: f64,
    pub sigma: f64,
}

/// Dimension 48
#[derive(Clone, Copy)]
pub struct PearsonCorrelation {
    pub mean_x: f64,
    pub mean_y: f64,
    pub n: f64,
    pub d: f64, // isto também dá a variança
    pub e: f64,
    pub k: f64, // nr de elementos
}

impl PearsonCorrelation {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self { mean_x: 0., mean_y: 0., n: 0., d: 0., e: 0., k: 0. }
    }

    #[inline]
    pub fn next(&mut self, x: f64, y: f64, row_nr: f64) {
        assert!(row_nr > 0.);
        let new_mean_x = self.mean_x + (x - self.mean_x) / row_nr;
        let new_mean_y = self.mean_y + (y - self.mean_y) / row_nr;
        self.n += (x - self.mean_x) * (y - new_mean_y);
        self.d += (x - self.mean_x) * (x - new_mean_x);
        self.e += (y - self.mean_y) * (y - new_mean_y);
        self.mean_x = new_mean_x;
        self.mean_y = new_mean_y;
        self.k = row_nr;
    }

    #[inline]
    pub fn corr(&self) -> f64 {
        assert!((self.d > 0.) && (self.e > 0.));
        self.n / (self.d.sqrt() * self.e.sqrt())
    }

    #[inline]
    pub fn mean(&self) -> f64 {
        self.mean_x
    }

    #[inline]
    pub fn var(&self) -> f64 {
        self.d / self.k
    }

    #[allow(dead_code)]
    // é para ser usada manualmente para futura expansão
    #[inline]
    pub fn var_corr(&self) -> f64 {
        self.d / (self.k - 1.)
    }

    #[inline]
    pub fn sigma(&self) -> f64 {
        f64::sqrt(self.var())
    }

    #[allow(dead_code)]
    // é para ser usada manualmente para futura expansão
    #[inline]
    pub fn sigma_corr(&self) -> f64 {
        f64::sqrt(self.var_corr())
    }
}
