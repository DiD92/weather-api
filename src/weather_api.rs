use log;

use crate::api_models;

const BASE_API_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

pub async fn get_weather_data_for(
    city_id: u32,
    temp_unit: char,
    api_key: &str,
) -> Option<api_models::APIResponse> {
    let request_url = format!("{}?id={}&appid={}", BASE_API_URL, city_id, api_key);

    let request = reqwest::get(&request_url).await;

    match request {
        Ok(response) => match response.json::<api_models::APIResponse>().await {
            Ok(json) => Some(json),
            Err(err) => {
                log::error!("{}", err);
                None
            }
        },
        Err(err) => {
            log::error!("{}", err);
            None
        }
    }
}
