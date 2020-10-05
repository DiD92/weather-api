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

pub struct APIClient {
    pub client: reqwest::Client,
    api_key: String,
}

impl APIClient {
    pub const BASE_API_URL: &'static str = "https://api.openweathermap.org/data/2.5/onecall??";

    pub fn build(api_key: String) -> Self {
        APIClient {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    const CURRENT_WEATHER_EXCLUDE: &'static str = "minutely,hourly,daily,alerts";

    pub async fn query_current_weather(
        &self,
        city_lat: f32,
        city_lon: f32,
        temperature_units: crate::api_models::TemperatureFormat,
    ) -> Result<APIResponse, reqwest::Error> {
        log::debug!(
            "Querying OpenWeatherMap API for coords - ({},{})",
            city_lat,
            city_lon
        );

        self.perform_query(
            city_lat,
            city_lon,
            temperature_units,
            &APIClient::CURRENT_WEATHER_EXCLUDE,
        )
        .await
    }

    const FORECAST_WEATHER_EXCLUDE: &'static str = "current,minutely,daily,alerts";

    pub async fn query_forecast_weather(
        &self,
        city_lat: f32,
        city_lon: f32,
        temperature_units: crate::api_models::TemperatureFormat,
    ) -> Result<APIResponse, reqwest::Error> {
        log::debug!(
            "Querying OpenWeatherMap API for coords - ({},{})",
            city_lat,
            city_lon
        );

        self.perform_query(
            city_lat,
            city_lon,
            temperature_units,
            &APIClient::FORECAST_WEATHER_EXCLUDE,
        )
        .await
    }

    async fn perform_query(
        &self,
        city_lat: f32,
        city_lon: f32,
        temperature_units: crate::api_models::TemperatureFormat,
        exclude_set: &str,
    ) -> Result<APIResponse, reqwest::Error> {
        let query_params = &[
            ("appid", &self.api_key),
            ("lat", &city_lat.to_string()),
            ("lon", &city_lon.to_string()),
            ("exclude", &exclude_set.to_owned()),
            ("units", &temperature_units.to_string()),
        ];

        let api_request = self
            .client
            .get(APIClient::BASE_API_URL)
            .query(query_params)
            .send()
            .await?
            .json::<APIResponse>()
            .await?;

        Ok(api_request)
    }
}

#[cfg(test)]
mod test_api_client {
    use super::*;

    #[actix_rt::test]
    async fn check_api_response() {
        // This key will not work, but we can at least get a
        // reply from the API
        let dummy_key = "aa";

        let city_coords: (f32, f32) = (1.0, 1.0);
        let temperature_fmt = crate::api_models::TemperatureFormat::Metric;

        let client = APIClient::build(dummy_key.to_owned());

        let query_result = client
            .query_current_weather(city_coords.0, city_coords.1, temperature_fmt)
            .await;

        assert!(query_result.is_ok());
    }

    #[actix_rt::test]
    async fn check_proper_api_response() {
        let open_weather_env_var = "OPENWEATHER_API_KEY";

        let api_key = std::env::var(open_weather_env_var);

        assert!(api_key.is_ok(), "OpenWeatherMap api key not found in env!");

        let api_key = api_key.unwrap();
        let city_coords: (f32, f32) = (34.940079, 36.321911); // Coords for city_id 2960
        let temperature_fmt = crate::api_models::TemperatureFormat::Metric;

        let client = APIClient::build(api_key.to_owned());

        let query_result = client
            .query_current_weather(city_coords.0, city_coords.1, temperature_fmt)
            .await;

        assert!(query_result.is_ok());

        let api_response = query_result.unwrap();

        assert!(api_response.current.is_some());
    }
}
