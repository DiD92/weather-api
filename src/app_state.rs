use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{api::APIResponse, state::*};
use crate::weather_api::APIClient;

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

pub struct AppState {
    pub api_client: APIClient,
    pub city_db: HashMap<(String, String), CityEntry>,
    api_cache: HashMap<CacheKey, CachedElement<APIResponse>>,
}

impl AppState {
    pub const CACHE_EXPIRY_MILIS: u128 = 600_000; // 10 minutes

    pub fn build(api_key: String, city_list: Vec<City>) -> Self {
        AppState {
            api_cache: HashMap::new(),
            api_client: crate::weather_api::APIClient::build(api_key),
            city_db: AppState::init_hash_table(city_list),
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

                let cache = CachedElement::new(response, AppState::CACHE_EXPIRY_MILIS);

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

    use crate::models::{api::WeatherCurrent, request::TemperatureFormat};

    #[test]
    fn check_cache_storage() {
        let mut app_state = AppState::build("11".into(), vec![]);

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
            current: Some(WeatherCurrent {
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
