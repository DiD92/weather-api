use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::weather_api::APIResponse;

#[derive(Deserialize, Serialize)]
pub struct City {
    pub id: u32,
    pub lat: f32,
    pub lon: f32,
    pub name: String,
    #[serde(rename(deserialize = "ctry"))]
    pub country: String,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureFormat {
    Metric,
    Imperial,
    Standard,
}

impl Display for TemperatureFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TemperatureFormat::Metric => write!(f, "metric"),
            TemperatureFormat::Imperial => write!(f, "imperial"),
            TemperatureFormat::Standard => write!(f, "standard"),
        }
    }
}

fn deserialize_from_str<'de, D>(deserializer: D) -> Result<TemperatureFormat, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;

    s.make_ascii_lowercase();

    match s.as_ref() {
        "f" | "fahrenheit" => Ok(TemperatureFormat::Imperial),
        "c" | "celsius" => Ok(TemperatureFormat::Metric),
        "k" | "kelvin" => Ok(TemperatureFormat::Standard),
        &_ => {
            log::warn!("Invalid temperature parameter supplied - {}", s);
            Err(Error::custom("Invalid temperature parameter."))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestBody {
    pub city_query: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    #[serde(rename = "units")]
    pub temperature_unit: TemperatureFormat,
}

#[derive(Deserialize, Serialize)]
pub struct RequestResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<ResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
}

impl RequestResponse {
    pub fn build_success(api_response: APIResponse) -> Self {
        RequestResponse {
            success: true,
            data: Some(ResponseData::Success(api_response)),
            msg: None,
        }
    }

    pub fn build_failure(failure_msg: String) -> Self {
        RequestResponse {
            success: false,
            data: None,
            msg: Some(failure_msg),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseData {
    Success(APIResponse),
    Failure(String),
}

pub struct CachedElement<T> {
    pub element: T,
    // Epoch in which the element expires
    pub expires_at: u128,
}

impl<T> CachedElement<T> {
    pub fn new(element: T, object_expiry_milis: u128) -> Self {
        Self {
            element,
            expires_at: Self::generate_expiry_time(object_expiry_milis),
        }
    }

    pub fn has_expired(&self) -> bool {
        let current_epoch = CachedElement::<T>::generate_expiry_time(0);
        current_epoch >= self.expires_at
    }

    fn generate_expiry_time(expiry_milis: u128) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            + expiry_milis
    }
}

#[derive(Copy, Clone)]
pub struct CityEntry {
    pub city_id: u32,
    pub city_lat: f32,
    pub city_lon: f32,
}

impl CityEntry {
    pub fn from(city_id: u32, city_lat: f32, city_lon: f32) -> Self {
        CityEntry {
            city_id,
            city_lat,
            city_lon,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub struct CacheKey {
    pub city_id: u32,
    pub temperature_fmt: TemperatureFormat,
    pub req_type: crate::RequestType,
}

impl CacheKey {
    pub fn from(
        city_id: u32,
        temperature_fmt: TemperatureFormat,
        req_type: crate::RequestType,
    ) -> Self {
        CacheKey {
            city_id,
            temperature_fmt,
            req_type,
        }
    }
}

pub struct APPState {
    pub api_client: crate::weather_api::APIClient,
    pub city_db: HashMap<(String, String), CityEntry>,
    api_cache: HashMap<CacheKey, CachedElement<APIResponse>>,
}

impl APPState {
    pub const CACHE_EXPIRY_MILIS: u128 = 600_000; // 10 minutes

    pub fn build(api_key: String, city_list: Vec<City>) -> Self {
        Self {
            api_cache: HashMap::new(),
            api_client: crate::weather_api::APIClient::build(api_key),
            city_db: APPState::init_hash_table(city_list),
        }
    }

    pub fn cache_response(
        &mut self,
        cache_key: CacheKey,
        response: APIResponse,
    ) -> Result<(), String> {
        if response.current.is_some() || response.hourly.is_some() {
            if !self.check_and_clear_cache(&cache_key) {
                log::debug!("Generating cache for api response - {}", &cache_key.city_id);

                let cache = CachedElement::new(response, APPState::CACHE_EXPIRY_MILIS);

                let _ = self.api_cache.insert(cache_key, cache);

                return Ok(());
            }

            log::warn!(
                "Tried to cache already cached api response for id - {}",
                &cache_key.city_id
            );

            Err("APIResponse is already cached!".into())
        } else {
            Err("APIResponse doesn't contain valid data!".into())
        }
    }

    pub fn get_cache_for(&mut self, cache_key: &CacheKey) -> Option<&APIResponse> {
        if self.check_and_clear_cache(cache_key) {
            return Some(&self.api_cache.get(cache_key).unwrap().element);
        }

        None
    }

    pub fn has_valid_cache_for(&self, cache_key: &CacheKey) -> bool {
        match self.api_cache.get(cache_key) {
            Some(cache) => !cache.has_expired(),
            None => false,
        }
    }

    fn check_and_clear_cache(&mut self, cache_key: &CacheKey) -> bool {
        match self.api_cache.get(cache_key) {
            Some(cache) => {
                if cache.has_expired() {
                    self.api_cache.remove(&cache_key);
                    return false;
                }

                true
            }
            None => false,
        }
    }

    fn init_hash_table(city_list: Vec<City>) -> HashMap<(String, String), CityEntry> {
        city_list
            .into_iter()
            .map(|entry| {
                (
                    (entry.name, entry.country),
                    CityEntry::from(entry.id, entry.lat, entry.lon),
                )
            })
            .collect()
    }

    pub fn get_city_keys_for_query(&self, city_query: &str) -> Option<CityEntry> {
        let query_parts = city_query.split(',').collect::<Vec<&str>>();

        if query_parts.len() == 2 {
            let query_key = (query_parts[0].to_string(), query_parts[1].to_string());
            self.city_db.get(&query_key).copied()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test_cached_element {
    use super::*;

    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn check_no_cache_config() {
        let cached_obj = CachedElement::new(10, 0);

        assert!(cached_obj.has_expired());
    }

    #[test]
    fn check_cache_persistence() {
        let cached_obj = CachedElement::new(10, 1000);
        assert!(!cached_obj.has_expired());

        sleep(Duration::from_millis(1100));

        assert!(cached_obj.has_expired());
    }
}

#[cfg(test)]
mod test_app_state {
    use super::*;

    #[test]
    fn check_cache_storage() {
        let mut app_state = APPState::build("11".into(), vec![]);

        let cache_key = CacheKey::from(
            1,
            TemperatureFormat::Metric,
            crate::RequestType::CurrentWeather,
        );

        let api_response = APIResponse {
            lat: None,
            lon: None,
            cod: None,
            message: None,
            current: None,
            hourly: None,
        };

        assert!(!app_state.has_valid_cache_for(&cache_key));

        assert!(app_state.cache_response(cache_key, api_response).is_err());

        assert!(!app_state.has_valid_cache_for(&cache_key));

        let api_response = APIResponse {
            lat: None,
            lon: None,
            cod: None,
            message: None,
            current: Some(crate::weather_api::WeatherCurrent {
                dt: 1,
                sunrise: 1,
                sunset: 1,
                temp: 0.0,
                feels_like: 0.0,
                pressure: 1,
                humidity: 1,
                dew_point: 0.0,
                uvi: 0.0,
                clouds: 1,
                visibility: 1,
                wind_speed: 0.0,
                wind_deg: 1,
                conditions: None,
            }),
            hourly: None,
        };

        assert!(app_state.cache_response(cache_key, api_response).is_ok());

        assert!(app_state.has_valid_cache_for(&cache_key));
    }
}
