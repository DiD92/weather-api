use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::Logger, web};
use env_logger::Env;
use std::sync::{Arc, Mutex};

mod api_models;
mod utils;
mod weather_api;

#[get("/weather")]
async fn current_weather_route(
    data: web::Data<Arc<Mutex<api_models::APPState>>>,
    body: web::Json<api_models::RequestBody>,
) -> impl Responder {
    let mut app_state = data.lock().unwrap();

    if let Some(city_keys) = app_state.get_city_keys_for_query(&body.city_query) {
        let cache_key = (city_keys.0, body.temperature_unit);

        if app_state.has_valid_cache_for(&cache_key) {
            let cached_response = app_state.get_cache_for(&cache_key).unwrap();
            HttpResponse::Ok().json(api_models::RequestResponse::build_success(
                cached_response.to_owned(),
            ))
        } else {
            match app_state
                .api_client
                .query_current_weather(city_keys.1, city_keys.2, cache_key.1)
                .await
            {
                Ok(response) => {
                    if response.cod.is_some() && response.cod.unwrap() != 200 {
                        HttpResponse::Ok().json(api_models::RequestResponse::build_failure(
                            response.message.unwrap(),
                        ))
                    } else {
                        if let Err(msg) =
                            app_state.cache_response(city_keys.0, response.clone(), cache_key.1)
                        {
                            log::warn!(
                                "Failed to created cache for ({}|{}) - {}",
                                cache_key.0,
                                cache_key.1,
                                msg
                            );
                        }

                        HttpResponse::Ok()
                            .json(api_models::RequestResponse::build_success(response))
                    }
                }
                Err(err) => HttpResponse::Ok()
                    .json(api_models::RequestResponse::build_failure(err.to_string())),
            }
        }
    } else {
        HttpResponse::Ok().json(api_models::RequestResponse::build_failure(format!(
            "No valid city_id found for query {}",
            &body.city_query
        )))
    }
}

#[get("/forecast")]
async fn weather_forecast_route(
    data: web::Data<Arc<Mutex<api_models::APPState>>>,
    body: web::Json<api_models::RequestBody>,
) -> impl Responder {
    let mut app_state = data.lock().unwrap();

    if let Some(city_keys) = app_state.get_city_keys_for_query(&body.city_query) {
        let cache_key = (city_keys.0, body.temperature_unit);

        if app_state.has_valid_cache_for(&cache_key) {
            let cached_response = app_state.get_cache_for(&cache_key).unwrap();
            HttpResponse::Ok().json(api_models::RequestResponse::build_success(
                cached_response.to_owned(),
            ))
        } else {
            match app_state
                .api_client
                .query_forecast_weather(city_keys.1, city_keys.2, cache_key.1)
                .await
            {
                Ok(response) => {
                    if response.cod.is_some() && response.cod.unwrap() != 200 {
                        HttpResponse::Ok().json(api_models::RequestResponse::build_failure(
                            response.message.unwrap(),
                        ))
                    } else {
                        if let Err(msg) =
                            app_state.cache_response(city_keys.0, response.clone(), cache_key.1)
                        {
                            log::warn!(
                                "Failed to created cache for ({}|{}) - {}",
                                cache_key.0,
                                cache_key.1,
                                msg
                            );
                        }

                        HttpResponse::Ok()
                            .json(api_models::RequestResponse::build_success(response))
                    }
                }
                Err(err) => HttpResponse::Ok()
                    .json(api_models::RequestResponse::build_failure(err.to_string())),
            }
        }
    } else {
        HttpResponse::Ok().json(api_models::RequestResponse::build_failure(format!(
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
            let app_state = api_models::APPState::build(api_key, city_db);

            let data = web::Data::new(Arc::new(Mutex::new(app_state)));

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
