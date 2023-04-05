use thiserror::*;

#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Error getting weather.")]
    GettingWeather,
    #[error("Error getting pressure and temperature history.")]
    GettingPressureAndTemperatureHistory,
    #[error("Error getting wind history.")]
    GettingWindHistory,
    #[error("Error inserting daily measures.")]
    CantInsertDailyMeasures,
    #[error("No aggregated daily measures for date: {}.", 0)]
    NoAggregatedDailyMeasuresForSelectedDate(String),
    #[error("Error calling url.")]
    CallingURL(String)
}

pub type WeatherResult<T> = Result<T, WeatherError>;
