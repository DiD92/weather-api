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

    if let Some(city_id) = app_state.get_city_id_for_query(&body.city_query) {
        let query_tuple = (city_id, body.temperature_unit);

        if app_state.has_valid_cache_for(&query_tuple) {
            let cached_response = app_state.get_cache_for(&query_tuple).unwrap();
            HttpResponse::Ok().json(api_models::RequestResponse::build_success(
                cached_response.to_owned(),
            ))
        } else {
            match app_state
                .api_client
                .query(query_tuple.0, query_tuple.1)
                .await
            {
                Ok(response) => {
                    if response.cod != 200 {
                        return HttpResponse::Ok().json(
                            api_models::RequestResponse::build_failure(response.message.unwrap()),
                        );
                    }

                    if let Err(msg) = app_state.cache_response(response.clone(), query_tuple.1) {
                        log::warn!(
                            "Failed to created cache for ({}|{}) - {}",
                            query_tuple.0,
                            query_tuple.1,
                            msg
                        );
                    }

                    HttpResponse::Ok().json(api_models::RequestResponse::build_success(response))
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
