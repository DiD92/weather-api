use serde::{de::Error, de::Visitor, Deserialize, Deserializer, Serialize};

fn deserialize_response_code<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct CodeVisitor;

    impl<'de> Visitor<'de> for CodeVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "status code either in string or u32 format")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match value.parse::<u32>() {
                Ok(num) => Ok(num),
                Err(_) => Err(E::custom(format!("Invalid status code - {}", value))),
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match value.parse::<u32>() {
                Ok(num) => Ok(num),
                Err(_) => Err(E::custom(format!("Invalid status code - {}", value))),
            }
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value as u32)
        }
    }

    deserializer.deserialize_any(CodeVisitor)
}

#[derive(Deserialize, Serialize, Clone)]
pub struct APIResponse {
    #[serde(skip_serializing)]
    #[serde(deserialize_with = "deserialize_response_code")]
    pub cod: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(deserialize = "main"))]
    pub details: Option<WeatherDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(deserialize = "weather"))]
    pub conditions: Option<Vec<WeatherCondition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind: Option<WeatherWind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherCondition {
    #[serde(rename(deserialize = "main"))]
    pub condition: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherDetails {
    pub temp: f32,
    pub pressure: u32,
    pub humidity: u32,
    pub temp_min: f32,
    pub temp_max: f32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WeatherWind {
    pub speed: f32,
    #[serde(rename(deserialize = "deg"))]
    pub degrees: u32,
}

pub struct APIClient {
    pub client: reqwest::Client,
    api_key: String,
}

impl APIClient {
    pub const BASE_API_URL: &'static str = "https://api.openweathermap.org/data/2.5/weather?";

    pub fn build(api_key: String) -> Self {
        APIClient {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn query(
        &self,
        city_id: u32,
        temperature_units: crate::api_models::TemperatureFormat,
    ) -> Result<APIResponse, reqwest::Error> {
        let query_params = &[
            ("appid", &self.api_key),
            ("id", &city_id.to_string()),
            ("units", &temperature_units.to_string()),
        ];

        log::debug!("Querying OpenWeatherMap API for city id - {}", city_id);

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

        let city_id = 2960;
        let temperature_fmt = crate::api_models::TemperatureFormat::Metric;

        let client = APIClient::build(dummy_key.to_owned());

        let query_result = client.query(city_id, temperature_fmt).await;

        assert!(query_result.is_ok());
    }

    #[actix_rt::test]
    async fn check_proper_api_response() {
        let open_weather_env_var = "OPENWEATHER_API_KEY";

        let api_key = std::env::var(open_weather_env_var);

        assert!(api_key.is_ok(), "OpenWeatherMap api key not found in env!");

        let api_key = api_key.unwrap();
        let city_id = 2960;
        let temperature_fmt = crate::api_models::TemperatureFormat::Metric;

        let client = APIClient::build(api_key.to_owned());

        let query_result = client.query(city_id, temperature_fmt).await;

        assert!(query_result.is_ok());

        let api_response = query_result.unwrap();

        assert_eq!(api_response.cod, 200);
        assert_eq!(api_response.id.unwrap(), city_id);
    }
}
