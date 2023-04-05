// The MIT License (MIT)
//
// Copyright (c) 2018 Nathan Osman
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

use crate::app_time::ctrl_time::*;

const J2000: f64 = 2451545.;
const DEGREE: f64 = 0.0174532925199433;
pub const SECONDS_IN_A_DAY: f64 = 86400.;
const UNIX_EPOCH_JULIAN_DAY: f64 = 2440587.5;

/// Calculates the sunrise and sunset times for the given location and date. <br>
/// <br>
/// Returns: (Sunrise, Sunset)
#[inline]
pub fn sun_times(date: CtrlTime, latitude: f64, longitude: f64, elevation: f64) -> (CtrlTime, CtrlTime) {
    let mut date = date.as_utc_date_time_e();

    date.hour = 12;
    date.min = 0;
    date.sec = 0;
    date.nanos = 0;

    let day: f64 = unix_to_julian(CtrlTime::from_utc_date_time_e(&date).ux_ts() as i64) - longitude / 360.;

    let mut solar_anomaly: f64 = (357.5291 + 0.98560028 * (day - J2000)) % 360.;
    if solar_anomaly < 0. {
        solar_anomaly += 360.;
    }

    let anomaly_in_rad: f64 = solar_anomaly * DEGREE;
    let anomaly_2_sin = f64::sin(2. * anomaly_in_rad);
    let anomaly_3_sin = f64::sin(3. * anomaly_in_rad);
    let equation_of_center: f64 = 1.9148 * f64::sin(anomaly_in_rad) + 0.02 * anomaly_2_sin + 0.0003 * anomaly_3_sin;

    let argument_of_perihelion = (102.93005 + 0.3179526 * (day - 2451545.) / 36525.) % 360.;
    let ecliptic_longitude: f64 = (solar_anomaly + equation_of_center + 180. + argument_of_perihelion + 360.) % 360.;
    let solar_transit: f64 = day + (0.0053 * f64::sin(solar_anomaly * DEGREE) - 0.0069 * f64::sin(2. * ecliptic_longitude * DEGREE));
    let declination: f64 = f64::asin(f64::sin(ecliptic_longitude * DEGREE) * 0.39779) / DEGREE;

    let elevation_correction = -2.076 * (f64::sqrt(elevation)) / 60.0;

    let latitude_rad = latitude * DEGREE;
    let declination_rad = declination * DEGREE;
    // let old_factor = -0.01449;
    let new_factor = f64::sin((-0.83 + elevation_correction).to_radians());
    let numerator = new_factor - f64::sin(latitude_rad) * f64::sin(declination_rad);
    let denominator = f64::cos(latitude_rad) * f64::cos(declination_rad);
    let hour_angle: f64 = f64::acos(numerator / denominator) / DEGREE;

    let frac: f64 = hour_angle / 360.;

    let ux_sunrise = julian_to_unix(solar_transit - frac) as u64;
    let ux_sunset = julian_to_unix(solar_transit + frac) as u64;
    (CtrlTime::from_ux_ts(ux_sunrise), CtrlTime::from_ux_ts(ux_sunset))
}

/// Converts a unix timestamp to a Julian day.
#[inline]
pub fn unix_to_julian(timestamp: i64) -> f64 {
    timestamp as f64 / SECONDS_IN_A_DAY + UNIX_EPOCH_JULIAN_DAY
}

/// Converts a Julian day to a unix timestamp.
#[inline]
pub fn julian_to_unix(day: f64) -> i64 {
    ((day - UNIX_EPOCH_JULIAN_DAY) * SECONDS_IN_A_DAY) as i64
}

#[cfg(test)]
mod tests {
    use crate::app_time::ctrl_time::*;

    use crate::app_time::sunrise::sun_times;

    use super::UNIX_EPOCH_JULIAN_DAY;

    #[test]
    fn test_prime_meridian() {
        let d = CtrlTime::from_utc_parts(1970, 1, 1, 0, 0, 0);

        assert!(sun_times(d, 0., 0., 0.) == (CtrlTime::from_ux_ts(21595 as u64), CtrlTime::from_ux_ts(65227)))
    }

    #[test]
    fn test_unix_to_julian() {
        assert_eq!(super::unix_to_julian(0), UNIX_EPOCH_JULIAN_DAY)
    }

    #[test]
    fn test_julian_to_unix() {
        assert_eq!(super::julian_to_unix(UNIX_EPOCH_JULIAN_DAY), 0)
    }
}
