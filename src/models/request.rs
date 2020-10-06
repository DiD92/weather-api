use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::models::api::APIResponse;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum RequestType {
    CurrentWeather,
    WeatherForecast,
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
