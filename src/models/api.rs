use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct APIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lon: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cod: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<WeatherCurrent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly: Option<Vec<WeatherHourly>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherCurrent {
    pub dt: u32,
    pub sunrise: u32,
    pub sunset: u32,
    pub temp: f32,
    pub feels_like: f32,
    pub pressure: u32,
    pub humidity: u32,
    pub dew_point: f32,
    pub uvi: f32,
    pub clouds: u32,
    pub visibility: u32,
    pub wind_speed: f32,
    pub wind_deg: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(deserialize = "weather"))]
    pub conditions: Option<Vec<WeatherCondition>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherCondition {
    #[serde(rename(deserialize = "main"))]
    pub condition: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherHourly {
    pub dt: u32,
    pub temp: f32,
    pub feels_like: f32,
    pub pressure: u32,
    pub humidity: u32,
    pub dew_point: f32,
    pub clouds: u32,
    pub visibility: u32,
    pub wind_speed: f32,
    pub wind_deg: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(deserialize = "weather"))]
    pub conditions: Option<Vec<WeatherCondition>>,
    pub pop: f32,
}
