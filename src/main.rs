use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::Logger, web};
use env_logger::Env;
use std::sync::{Arc, Mutex};

mod app_state;
mod models;
mod utils;
mod weather_api;

use crate::models::{api::APIResponse, request::*, state::CacheKey};

type SharedState = web::Data<Arc<Mutex<app_state::APPState>>>;
type InboundRequest = web::Json<RequestBody>;

#[get("/weather")]
async fn current_weather_route(data: SharedState, body: InboundRequest) -> impl Responder {
    process_route(data, body, RequestType::CurrentWeather).await
}

#[get("/forecast")]
async fn weather_forecast_route(data: SharedState, body: InboundRequest) -> impl Responder {
    process_route(data, body, RequestType::WeatherForecast).await
}

async fn process_route(
    data: SharedState,
    body: InboundRequest,
    request_type: RequestType,
) -> impl Responder {
    let mut app_state = data.lock().unwrap();

    if let Some(city_keys) = app_state.get_city_keys_for_query(&body.city_query) {
        let cache_key = CacheKey::from(city_keys.city_id, body.temperature_unit, request_type);

        if app_state.has_valid_cache_for(&cache_key) {
            let cached_response = app_state.get_cache_for(&cache_key).unwrap();
            HttpResponse::Ok().json(RequestResponse::build_success(cached_response.to_owned()))
        } else {
            let api_result: Result<APIResponse, reqwest::Error>;

            match request_type {
                RequestType::CurrentWeather => {
                    api_result = app_state
                        .api_client
                        .query_current_weather(
                            city_keys.city_lat,
                            city_keys.city_lon,
                            body.temperature_unit,
                        )
                        .await;
                }
                RequestType::WeatherForecast => {
                    api_result = app_state
                        .api_client
                        .query_forecast_weather(
                            city_keys.city_lat,
                            city_keys.city_lon,
                            body.temperature_unit,
                        )
                        .await;
                }
            }

            match api_result {
                Ok(response) => {
                    if response.cod.is_some() && response.cod.unwrap() != 200 {
                        HttpResponse::Ok()
                            .json(RequestResponse::build_failure(response.message.unwrap()))
                    } else {
                        if let Err(msg) = app_state.cache_response(cache_key, response.clone()) {
                            log::warn!(
                                "Failed to created cache for ({}|{:?}|{:?}) - {}",
                                cache_key.city_id,
                                cache_key.temperature_fmt,
                                cache_key.req_type,
                                msg
                            );
                        }

                        HttpResponse::Ok().json(RequestResponse::build_success(response))
                    }
                }
                Err(err) => {
                    HttpResponse::Ok().json(RequestResponse::build_failure(err.to_string()))
                }
            }
        }
    } else {
        HttpResponse::Ok().json(RequestResponse::build_failure(format!(
            "No valid city_id found for query {}",
            &body.city_query
        )))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if utils::is_app_running_in_prod() {
        env_logger::from_env(Env::default().default_filter_or("info")).init();
        log::info!("Starting server in production environment...");
    } else {
        env_logger::from_env(Env::default().default_filter_or("debug")).init();
        log::info!("Starting server in development environment...");
    }

    match (utils::get_api_key(), utils::load_city_db()) {
        (Some(api_key), Some(city_db)) => {
            let app_state = app_state::APPState::build(api_key, city_db);

            let data: SharedState = web::Data::new(Arc::new(Mutex::new(app_state)));

            HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i"))
                    .app_data(data.clone())
                    .service(current_weather_route)
                    .service(weather_forecast_route)
            })
            .bind("localhost:8080")?
            .run()
            .await
        }
        (_, _) => {
            log::error!("Errors found during server initialization, shutting down...");
            Ok(())
        }
    }
}
