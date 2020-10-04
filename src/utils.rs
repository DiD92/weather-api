use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use flate2::bufread::GzDecoder;

pub const APP_DEVELOPMENT_FLAG: &str = "WEATHER_API_SERVER_DEV";

pub fn is_app_running_in_prod() -> bool {
    std::env::var(APP_DEVELOPMENT_FLAG).is_ok()
}

pub const API_KEY_ENV_VAR: &str = "OPENWEATHER_API_KEY";

pub fn get_api_key() -> Option<String> {
    match std::env::var(API_KEY_ENV_VAR) {
        Ok(api_key) => Some(api_key),
        Err(err) => {
            log::error!("{}", format!("api key could not be loaded - {}", err));
            None
        }
    }
}

pub const CITY_DB_ENV_VAR: &str = "CITY_DATABASE_PATH";

pub const CITY_DB_FILENAME: &str = "cities_db.json.gz";

pub fn load_city_db() -> Option<Vec<crate::api_models::City>> {
    match std::env::var(CITY_DB_ENV_VAR) {
        Ok(db_path) => {
            let mut file_path = PathBuf::from(db_path);

            if file_path.is_dir() {
                file_path.push(CITY_DB_FILENAME);

                if file_path.is_file() {
                    match File::open(file_path) {
                        Ok(file) => {
                            let reader = GzDecoder::new(BufReader::new(file));

                            match serde_json::from_reader::<_, Vec<crate::api_models::City>>(reader)
                            {
                                Ok(city_list) => {
                                    log::info!(
                                        "{} entries loaded from cities database",
                                        &city_list.len()
                                    );
                                    Some(city_list)
                                }
                                Err(err) => {
                                    log::error!(
                                        "{}",
                                        format!("Error parsing database file - {}", err)
                                    );
                                    None
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("{}", format!("Error opening database file - {}", err));
                            None
                        }
                    }
                } else {
                    log::error!(
                        "{}",
                        format!(
                            "{} could not be found in {} is not a valid directory!",
                            CITY_DB_FILENAME, CITY_DB_ENV_VAR
                        )
                    );
                    None
                }
            } else {
                log::error!(
                    "{}",
                    format!(
                        "The path pointed by {} is not a valid directory!",
                        CITY_DB_ENV_VAR
                    )
                );
                None
            }
        }
        Err(err) => {
            log::error!("{}", format!("city database could not be loaded - {}", err));
            None
        }
    }
}
