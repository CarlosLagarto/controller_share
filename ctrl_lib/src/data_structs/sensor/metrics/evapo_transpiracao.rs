use crate::app_time::ctrl_time::*;
use crate::data_structs::sensor::stat_metric::{Metric, MAX_FEATURES};
use crate::services::weather::rain_pred::data_structs::DSRow;

/// Penman-Monteith equation
/// https://www.fao.org/3/x0490e/x0490e08.htm#TopOfPage
/// wind_speed (km/s) deve ser recolhida pelo menos 2m acima do solo
/// atmospheric_pressure mb 1013 milibar = 101.3 KPa (kilo pascal)
///
const SOLAR_CONSTANT: f64 = 0.0820;
const STEFAN_BOLTZMAN_CONSTANT: f64 = 0.000000004903;
const ALBEDO: f64 = 0.23;
// os 2 parametros seguintes depende de poeiras do ar, nebulosidade, etc.
// Na ausencia destes valores estes são os valores recomendados para a estimativa
// equation 28
const FRACTION_EXTRATERRESTRIAL_RADIATION_OVERCAST: f64 = 0.25; //
const FRACTION_EXTRATERRESTRIAL_RADIATION_CLEAR: f64 = 0.5;
const TWO_PI_TIMES_365: f64 = 0.017214206;
const MINUTES_PER_PI: f64 = 458.3662361;
const WIND_CONV_FACTOR: f64 = 0.207764188;
const DIV_100_TO_MUL: f64 = 0.01; //1 / 100.
const DIV_2_TO_MUL: f64 = 0.5;

/// Dimension = 80
#[derive(Clone, Default)]
pub struct EtData {
    pub lat: f64,
    pub time: CtrlTime,
    pub avg_ws: f64,
    pub max_t: f64,
    pub min_t: f64,
    pub max_hr: f64,
    pub min_hr: f64,
    pub elev: f64,
    pub avg_hr: f64,
    pub avg_press: f64,
}

impl EtData {
    #[inline]
    pub fn from_ds_row(ds_row: &DSRow<MAX_FEATURES>, elev: f64, lat: f64, time: CtrlTime) -> Self {
        Self {
            avg_hr: ds_row[Metric::AvgHumidity as usize],
            avg_press: ds_row[Metric::AvgPressure as usize],
            avg_ws: ds_row[Metric::AvgWindSpeed as usize],
            max_hr: ds_row[Metric::MaxHumidity as usize],
            max_t: ds_row[Metric::MaxTemp as usize],
            min_hr: ds_row[Metric::MinHumidity as usize],
            min_t: ds_row[Metric::MinTemp as usize],
            elev,
            lat,
            time,
        }
    }

    #[inline]
    pub fn valid_et_data(row: &DSRow<MAX_FEATURES>) -> bool {
        !row[Metric::AvgHumidity as usize].is_nan()
            && !row[Metric::AvgPressure as usize].is_nan()
            && !row[Metric::AvgWindSpeed as usize].is_nan()
            && !row[Metric::MaxHumidity as usize].is_nan()
            && !row[Metric::MaxTemp as usize].is_nan()
            && !row[Metric::MinHumidity as usize].is_nan()
            && !row[Metric::MinTemp as usize].is_nan()
    }
}

const POTENTIAL_ERROR_FACTOR: f64 = 0.6; // 40% erro potencial máximo, quer dizer que vamos multiplicar o resultado por 0.6;

/// Accuracy - wikipedia
///
/// While the Penman-Monteith method is widely considered accurate for practical purposes and is recommended by the Food and Agriculture Organization
/// of the United Nations, errors when compared to direct measurement or other techniques can range from -9 to 40%. <br>
/// <br>
/// Por isto , como medida de poupar energia, decidi "cortar" a estimativa em 40%. <br>
/// <br>
/// Desta forma poupo energia, e a consequência potencial é regar um pouco menos do que o ideal, mas assumo que pela observação visual, isto vai sendo afinado,
/// e aumenta-se o valor de referência semanal se for o caso, ou o tempo de algum setor.  <br>
/// <br>
/// Num sistema em perfeito equilibrio, e com os sensores de humididade no terreno, pode-se construir um sistema que dê feedback e auto-regule os fatores,
/// mas isso será para uma outra iteração FUTURE:
pub fn daily_evapo_transpiration(et_data: EtData) -> f32 {
    let lat_rad = et_data.lat.to_radians();
    let day_number = et_data.time.year_day_number_e() as f64;
    let wind_speed_conv = et_data.avg_ws * WIND_CONV_FACTOR;
    let psycometric_constant = 0.0000665 * et_data.avg_press; // at sea level
    let nr_voltas_ao_sol = TWO_PI_TIMES_365 * day_number;
    let solar_declination = 0.409 * f64::sin(nr_voltas_ao_sol - 1.39);
    let sunset_hour_angle = f64::acos(-f64::tan(lat_rad) * f64::tan(solar_declination));

    let mean_t = (et_data.max_t + et_data.min_t) * 0.5;
    let mean_sat_vapour_pressure_max_t = 0.6108 * (f64::exp((17.27 * et_data.max_t) / (et_data.max_t + 237.3)));
    let mean_sat_vapour_pressure_min_t = 0.6108 * (f64::exp((17.27 * et_data.min_t) / (et_data.min_t + 237.3)));
    let mean_sat_vapour_pressure = (mean_sat_vapour_pressure_max_t + mean_sat_vapour_pressure_min_t) * DIV_2_TO_MUL;
    let actual_vapour_pressure =
        (mean_sat_vapour_pressure_min_t * (et_data.max_hr * DIV_100_TO_MUL) + mean_sat_vapour_pressure_max_t * (et_data.min_hr * DIV_100_TO_MUL)) * DIV_2_TO_MUL;

    // equation 23
    let inverse_relative_distance_earth_sun = 1. + (0.033 * f64::cos(nr_voltas_ao_sol));

    // equation 28
    let ra_extraterrestrial_radiation = MINUTES_PER_PI
        * SOLAR_CONSTANT
        * inverse_relative_distance_earth_sun
        * ((sunset_hour_angle * f64::sin(lat_rad) * f64::sin(solar_declination))
            + (f64::cos(lat_rad) * f64::cos(solar_declination) * (f64::sin(sunset_hour_angle))));

    // equation 35
    let rs_solar_radiation = (FRACTION_EXTRATERRESTRIAL_RADIATION_OVERCAST + FRACTION_EXTRATERRESTRIAL_RADIATION_CLEAR) * ra_extraterrestrial_radiation;
    // equation 38
    let rns = (1. - ALBEDO) * rs_solar_radiation;
    // equation 37
    let clear_sky_radiation = (0.75 + (0.00002 * et_data.elev)) * ra_extraterrestrial_radiation;
    // equation 39
    let stefan_boltzman_factor_min_t = STEFAN_BOLTZMAN_CONSTANT * f64::powi(et_data.min_t + 273.16, 4);
    let stefan_boltzman_factor_max_t = STEFAN_BOLTZMAN_CONSTANT * f64::powi(et_data.max_t + 273.16, 4);
    let stefan_boltzman_factor = (stefan_boltzman_factor_min_t + stefan_boltzman_factor_max_t) * DIV_2_TO_MUL;

    let air_humidity_correction = 0.34 - (0.14 * f64::sqrt(actual_vapour_pressure));

    let relative_shortwave_radiation = f64::max(rs_solar_radiation / clear_sky_radiation, 1.0);

    let cloudiness = (1.35 * relative_shortwave_radiation) - 0.35;

    let rnl = stefan_boltzman_factor * air_humidity_correction * cloudiness;
    // let rn = rns - rnl;
    let factor1 = 0.408 * (rns - rnl);
    // equation 13
    let mean_t_factor = mean_t + 237.3;
    let mean_sat_vapour_press = 0.6108 * f64::exp((17.27 * mean_t) / mean_t_factor);
    let slope_vapour_pressure = (4098. * mean_sat_vapour_press) / f64::powi(mean_t_factor, 2);
    // equation 54
    let actual_vapour_pressure = mean_sat_vapour_press * (et_data.avg_hr * DIV_100_TO_MUL);
    // equation 53
    let denominador = slope_vapour_pressure + (psycometric_constant * (1. + (0.34 * wind_speed_conv)));
    let factor2 = factor1 * slope_vapour_pressure / denominador;
    let factor3 = (900. / (mean_t + 273.)) * wind_speed_conv * (mean_sat_vapour_pressure - actual_vapour_pressure) * psycometric_constant / denominador;
    f32::max(((factor3 + factor2) * POTENTIAL_ERROR_FACTOR) as f32, 0.)
}
