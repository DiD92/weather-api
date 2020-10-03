use log;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone)]
pub struct APIResponse {
    pub cod: u16,
    pub id: Option<u32>,
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
        temperature_units: char,
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
        let temperature_fmt = 'C';

        let client = APIClient::build(dummy_key.to_owned());

        let query_result = client.query(city_id, temperature_fmt).await;

        assert!(query_result.is_ok());
    }

    #[actix_rt::test]
    async fn check_proper_api_response() {
        // This key will not work, but we can at least get a
        // reply from the API

        let open_weather_env_var = "OPENWEATHER_API_KEY";

        let api_key = std::env::var(open_weather_env_var);

        assert!(api_key.is_ok());

        let api_key = api_key.unwrap();
        let city_id = 2960;
        let temperature_fmt = 'C';

        let client = APIClient::build(api_key.to_owned());

        let query_result = client.query(city_id, temperature_fmt).await;

        assert!(query_result.is_ok());

        let api_response = query_result.unwrap();

        assert_eq!(api_response.cod, 200);
        assert_eq!(api_response.id.unwrap(), city_id);
    }
}
