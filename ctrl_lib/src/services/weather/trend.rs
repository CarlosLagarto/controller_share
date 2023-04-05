use arrayvec::ArrayVec;

// The following table lists values for t-distributions with ν degrees of freedom for a range of one-sided on the 95% column
// degrees of freedom larger tham 120, is always the same, the value at the 121 position
const T_INV_VALS: [f64; 121] = [
    6.314, 2.920, 2.353, 2.132, 2.015, 1.943, 1.895, 1.860, 1.833, 1.812, 1.796, 1.782, 1.771, 1.761, 1.753, 1.746, 1.740, 1.734, 1.729, 1.725, 1.721, 1.717,
    1.714, 1.711, 1.708, 1.706, 1.703, 1.701, 1.699, 1.697, 1.6957, 1.6944, 1.6931, 1.6918, 1.6905, 1.6892, 1.6879, 1.6866, 1.6853, 1.684, 1.6832, 1.6824,
    1.6816, 1.6808, 1.68, 1.6792, 1.6784, 1.6776, 1.6768, 1.676, 1.6755, 1.675, 1.6745, 1.674, 1.6735, 1.673, 1.6725, 1.672, 1.6715, 1.671, 1.67065, 1.6703,
    1.66995, 1.6696, 1.66925, 1.6689, 1.66855, 1.6682, 1.66785, 1.6675, 1.66715, 1.6668, 1.66645, 1.6661, 1.66575, 1.6654, 1.66505, 1.6647, 1.66435, 1.664,
    1.6638, 1.6636, 1.6634, 1.6632, 1.663, 1.6628, 1.6626, 1.6624, 1.6622, 1.662, 1.6618, 1.6616, 1.6614, 1.6612, 1.661, 1.6608, 1.6606, 1.6604, 1.6602, 1.660,
    1.6599, 1.6598, 1.6597, 1.6596, 1.6595, 1.6594, 1.6593, 1.6592, 1.6591, 1.659, 1.6589, 1.6588, 1.6587, 1.6586, 1.6585, 1.6584, 1.6583, 1.6582, 1.6581,
    1.658, 1.645,
];

#[inline]
pub const fn t_inv(degree_of_freedom: usize) -> f64 {
    if degree_of_freedom < 121 {
        T_INV_VALS[degree_of_freedom]
    } else {
        T_INV_VALS[120]
    }
}

#[inline]
#[rustfmt::skip]
pub const fn degrees_of_freedom(array_size: usize) -> usize {
    if array_size < 2 { 0 } else { array_size - 2 }
}

#[repr(u8)]
pub enum TrendC {
    Falling = 0,
    Steday = 1,
    Rising = 2,
}

const MAX_TREND_DATA: usize = 60; //1 hora de 60 leituras por hora
pub struct TrendA {
    data: ArrayVec<f64, MAX_TREND_DATA>,
}

impl TrendA {
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self { data: ArrayVec::new(), }
    }

    /// Se result > 0 trend rising <br>
    /// se result = 0 trend steady <br>
    /// Se result < 0 trend falling <br>
    /// https://gist.github.com/Paraphraser/c5609f85cc7ee6ecd03ce179fb7f7edb
    pub fn trend_analysis(&mut self, new_pressure: f64) -> f32 {
        if self.data.len() < MAX_TREND_DATA {
            self.data.push(new_pressure);
        } else {
            self.data.pop_at(0);
            self.data.push(new_pressure);
        }

        // Step 1 : calculate the straight line of best fit (least-squares linear regression)
        let mut sum_x = 0.0; // ∑(x)
        let mut sum_xx = 0.0; // ∑(x²)
        let mut sum_y = 0.0; // ∑(y)
        let mut sum_xy = 0.0; // ∑(xy)
        let len = self.data.len();

        // vou ver como se comporta com menos dados...., mas vamos testar
        if len < 2 {
            return 0.; //steady porque ainda não temos pontos suficientes
        }
        // we need n in lots of places and it's convenient as a double
        let n = 1.0 * len as f64; //PressureHistorySize;
        let mut y: f64;
        let mut x: f64;

        // iterate to calculate the above values
        for i in 0..len {
            x = 1.0 * i as f64;
            y = self.data[i];

            sum_x += x;
            sum_xx += x * x;
            sum_y += y;
            sum_xy += x * y;
        }

        // calculate the slope and intercept
        let slope = (sum_x * sum_y - n * sum_xy) / (sum_x * sum_x - n * sum_xx);
        let intercept = (sum_y - slope * sum_x) / n;

        /*
         * Step 2 : Perform an hypothesis test on the equation of the linear model to see whether, statistically, the available data
         *          contains sufficient evidence to conclude that the slope is non-zero.
         *
         *          Let beta1 = the slope of the regression line between fixed time intervals and pressure observations.
         *
         *          H0: β₁ = 0    (the slope is zero)
         *          H1: β₁ ≠ 0    (the slope is not zero)
         *
         *          The level of significance: α is 5% (0.05)
         *
         *          The test statistic is:
         *
         *              tObserved = (b₁ - β₁) / s_b₁
         *
         *          In this context, b₁ is the estimated slope of the linear model and β₁ the reference value from the hypothesis
         *          being tested. s_b₁ is the standard error of b₁.
         *
         *          From H0, β₁ = 0 so the test statistic simplifies to:
         *
         *              tObserved = b₁ / s_b₁
         *
         *          This is a two-tailed test so half of α goes on each side of the T distribution.
         *
         *          The degrees-of-freedom, ν, for the test is:
         *
         *              ν = n-2 = 6 - 2 = 4
         *
         *          The critical value (calculated externally using Excel or a graphics calculator) is:
         *
         *              -tCritical = invt(0.05/2,4) = -2.776445105
         *
         *          By symmetry:
         *
         *              +tCritical = abs(-tCritical)
         *
         *          The decision rule is:
         *              reject H0 if tObserved < -tCritical or tObserved > +tCritical
         *
         *          which can be simplified to:
         *              reject H0 if abs(tObserved) > +tCritical
         *
         *          Note that the value of +tCritical is carried in the global variable:
         *
         *              Critical_t_value
         *
         *          The next step is to calculate the test statistic but one of the inputs to that calculation is SSE, so we need that first.
         */
        let mut sse = 0.0; // ∑((y-ŷ)²)
        let mut residual: f64;
        // iterate
        for i in 0..len {
            y = self.data[i];
            residual = y - (intercept + slope * i as f64);
            sse += residual * residual;
        }

        // Now we can calculate the test statistic.
        // Note the use of the fabs() function below to force the result into the positive domain for comparison with Critical_t_value
        let t_observed = f64::abs(slope / (f64::sqrt(sse / (n - 2.0)) / f64::sqrt(sum_xx - sum_x * sum_x / n)));

        // Finally, make the decision and return a string summarising the conclusion.
        let critical_t_value = t_inv(degrees_of_freedom(self.data.len()));
        if t_observed > critical_t_value {
            // is tObserved further to the left or right than tCritical?
            slope as f32
        } else {
            // otherwise, the slope may be zero (statistically)
            0.
        }
    }
}
