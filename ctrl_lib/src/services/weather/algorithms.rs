// use arrayvec::ArrayVec;

/// station presure em hPa (mb / 1000)
/// temperature em celcius
/// altitude da estação em m
#[inline]
pub fn station_pressure_to_sea_pressure(pressure: f32, temperature: f32, altitude: f32) -> f32 {
    let altitude_factor = 0.0065 * altitude;
    pressure * f32::powf(1. - ((altitude_factor) / (temperature + altitude_factor + 273.15)), -5.257)
}

// a precisão deste calculo é a 0.35 ºC
#[inline]
pub fn dew_point(temperature: f64, relative_humidity: f64) -> f64 {
    let temp: f64 = temperature;
    let rh = relative_humidity;

    let alpha = f64::ln(rh / 100.) + 17.625 * temp / (243.04 + temp);
    (243.04 * alpha) / (17.625 - alpha)
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::{dew_point, station_pressure_to_sea_pressure};
    #[test]
    fn test_station_pressure() {
        let calc_press = station_pressure_to_sea_pressure(1.004, 32.5, 95.4) * 1000.;
        println!("calc press: {}", calc_press);
        assert!(f32::abs(calc_press - 1014.7544) < f32::EPSILON);
    }
    #[test]
    fn test_dew_point() {
        let dp = dew_point(31.0, 46.);
        println!("dew point: {}", dp);
        assert!(f64::abs(dp - 18.0) < 0.35);
    }
}
