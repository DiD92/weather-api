use serde::{Deserialize, Serialize};

use crate::models::request::{RequestType, TemperatureFormat};

#[derive(Deserialize, Serialize)]
pub struct City {
    pub id: u32,
    pub lat: f32,
    pub lon: f32,
    pub name: String,
    #[serde(rename(deserialize = "ctry"))]
    pub country: String,
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
    pub req_type: RequestType,
}

impl CacheKey {
    pub fn from(city_id: u32, temperature_fmt: TemperatureFormat, req_type: RequestType) -> Self {
        CacheKey {
            city_id,
            temperature_fmt,
            req_type,
        }
    }
}
