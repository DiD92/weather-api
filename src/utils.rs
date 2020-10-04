
pub const APP_DEVELOPMENT_FLAG: &str = "WEATHER_API_SERVER_DEV";

pub fn is_app_running_in_prod() -> bool {
    std::env::var(APP_DEVELOPMENT_FLAG).is_ok()
}

pub const API_KEY_ENV_VAR: &str = "OPENWEATHER_API_KEY";

pub fn get_api_key() -> Option<String> {
    match std::env::var(API_KEY_ENV_VAR) {
        Ok(api_key) => Some(api_key),
        Err(err) => {
            log::debug!("{}", format!("api key could not be loaded - {}", err));
            None
        }
    }
}
