use serde::{Deserialize, Serialize};

/// Dimension = 8
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AlrtThresholds {
    pub rain: f32, 
    pub wind: f32,
}

impl AlrtThresholds {

    #[inline]
    #[rustfmt::skip]
    pub fn is_rain_alert(&self, rain: f32) -> bool { rain >= self.rain }

    #[inline]
    #[rustfmt::skip]
    pub fn is_wind_alert(&self, wind: f32) -> bool { wind >= self.wind }

    #[inline]
    #[rustfmt::skip]
    pub fn is_weather_alert(&self, rain: f32, wind: f32) -> bool { self.is_rain_alert(rain) || self.is_wind_alert(wind) }
}
