use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{de::Error, Deserialize, Deserializer, Serialize};

fn deserialize_from_str<'de, D>(deserializer: D) -> Result<char, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;

    s.make_ascii_lowercase();

    match s.as_ref() {
        "f" | "fahrenheit" => Ok('f'),
        "c" | "celsius" => Ok('c'),
        &_ => {
            // TODO: Log error here
            return Err(Error::custom("Invalid temperature parameter."));
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestBody {
    pub city_id: u32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub temp_format: char,
}

#[derive(Deserialize, Serialize, Copy, Clone)]
pub struct APIResponse {
    pub cod: u16,
    pub id: Option<u32>,
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
            expires_at: Self::generate_expiry_time(object_expiry_milis)
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
            .as_millis() + expiry_milis
    }
}

pub struct APPState {
    pub api_key: String,
    api_cache: HashMap<u32, CachedElement<APIResponse>>,
}

impl APPState {

    pub const CACHE_EXPIRY_MILIS: u128 = 300_000;

    pub fn build(api_key: String) -> Self {
        Self {
            api_key,
            api_cache: HashMap::new(),
        }
    }

    pub fn cache_response(&mut self, response: APIResponse) -> Result<(), String> {
        match response.id {
            Some(city_id) => {
                if !self.check_and_clear_cache(city_id) {

                    let cache = CachedElement::new(response, APPState::CACHE_EXPIRY_MILIS);

                    let _ = self.api_cache.insert(city_id, cache);

                    return Ok(());
                }

                Err("APIResponse is already cached!".into())
            }
            None => Err("APIResponse doesn't contain identifier!".into()),
        }
    }

    pub fn get_cache_for(&mut self, cache_key: u32) -> Option<&APIResponse> {
        if self.check_and_clear_cache(cache_key) {
            return Some(&self.api_cache.get(&cache_key).unwrap().element);
        }

        None
    }

    pub fn has_valid_cache_for(&self, cache_key: u32) -> bool {
        match self.api_cache.get(&cache_key) {
            Some(cache) => !cache.has_expired(),
            None => false,
        }
    }

    fn check_and_clear_cache(&mut self, cache_key: u32) -> bool {
        match self.api_cache.get(&cache_key) {
            Some(cache) => {
                if cache.has_expired() {
                    self.api_cache.remove(&cache_key);
                    return false;
                }

                return true;
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

        let cache_key = 1;

        let api_response = APIResponse {cod: 200, id: Some(cache_key)};

        assert!(!app_state.has_valid_cache_for(cache_key));

        assert_eq!(app_state.cache_response(api_response), Ok(()));

        assert!(app_state.has_valid_cache_for(cache_key));
    }
}
