use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::weather_api::APIResponse;

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Copy, Clone)]
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
    pub city_id: u32,
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

pub struct APPState {
    pub api_client: crate::weather_api::APIClient,
    api_cache: HashMap<(u32, TemperatureFormat), CachedElement<APIResponse>>,
}

impl APPState {
    pub const CACHE_EXPIRY_MILIS: u128 = 300_000;

    pub fn build(api_key: String) -> Self {
        Self {
            api_cache: HashMap::new(),
            api_client: crate::weather_api::APIClient::build(api_key),
        }
    }

    pub fn cache_response(
        &mut self,
        response: APIResponse,
        temperature_unit_used: TemperatureFormat,
    ) -> Result<(), String> {
        match response.id {
            Some(city_id) => {
                if !self.check_and_clear_cache(&(city_id, temperature_unit_used)) {
                    log::debug!("Generating cache for api response - {}", city_id);

                    let cache = CachedElement::new(response, APPState::CACHE_EXPIRY_MILIS);

                    let _ = self
                        .api_cache
                        .insert((city_id, temperature_unit_used), cache);

                    return Ok(());
                }

                log::warn!(
                    "Tried to cache already cached api response for id - {}",
                    city_id
                );

                Err("APIResponse is already cached!".into())
            }
            None => Err("APIResponse doesn't contain identifier!".into()),
        }
    }

    pub fn get_cache_for(&mut self, cache_key: &(u32, TemperatureFormat)) -> Option<&APIResponse> {
        if self.check_and_clear_cache(cache_key) {
            return Some(&self.api_cache.get(cache_key).unwrap().element);
        }

        None
    }

    pub fn has_valid_cache_for(&self, cache_key: &(u32, TemperatureFormat)) -> bool {
        match self.api_cache.get(cache_key) {
            Some(cache) => !cache.has_expired(),
            None => false,
        }
    }

    fn check_and_clear_cache(&mut self, cache_key: &(u32, TemperatureFormat)) -> bool {
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
        let mut app_state = APPState::build("11".into());

        let cache_key = (1, TemperatureFormat::Metric);

        let api_response = APIResponse {
            cod: 200,
            id: Some(cache_key.0),
            name: None,
            message: None,
            conditions: None,
            details: None,
            wind: None,
        };

        assert!(!app_state.has_valid_cache_for(&cache_key));

        assert_eq!(app_state.cache_response(api_response, cache_key.1), Ok(()));

        assert!(app_state.has_valid_cache_for(&cache_key));
    }
}
