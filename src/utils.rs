use std::path::PathBuf;

use crate::models::state::City;

pub const APP_DEVELOPMENT_FLAG: &str = "WEATHER_API_SERVER_PROD";

pub fn is_app_running_in_prod() -> bool {
    std::env::var(APP_DEVELOPMENT_FLAG).is_ok()
}

pub const API_KEY_ENV_VAR: &str = "OPENWEATHER_API_KEY";

pub fn get_api_key() -> Option<String> {
    match std::env::var(API_KEY_ENV_VAR) {
        Ok(api_key) => Some(api_key),
        Err(err) => {
            log::error!("api key could not be loaded - {}", err);
            None
        }
    }
}

pub const CITY_DB_ENV_VAR: &str = "CITY_DATABASE_PATH";

pub const CITY_DB_FILENAME: &str = "cities_db.json";

pub fn load_city_db() -> Option<Vec<City>> {
    match std::env::var(CITY_DB_ENV_VAR) {
        Ok(db_path) => {
            let mut file_path = PathBuf::from(db_path);

            if !file_path.is_dir() {
                log::error!(
                    "Database path is not a valid directory - {}",
                    file_path.to_string_lossy()
                );
                return None;
            }

            file_path.push(CITY_DB_FILENAME);

            match std::fs::read_to_string(file_path) {
                Ok(str_content) => match serde_json::from_str::<Vec<City>>(&str_content) {
                    Ok(city_list) => {
                        log::info!("{} entries loaded from cities database", city_list.len());
                        Some(city_list)
                    }
                    Err(err) => {
                        log::error!("Error parsing database file - {}", err);
                        None
                    }
                },
                Err(err) => {
                    log::error!("Error reading database file - {}", err);
                    None
                }
            }
        }
        Err(err) => {
            log::error!("city database could not be loaded - {}", err);
            None
        }
    }
}
