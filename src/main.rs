use actix_web::{middleware::Logger, web};
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use std::sync::{Arc, Mutex};

mod api_models;
mod weather_api;

#[post("/weather")]
async fn current_weather_route(
    data: web::Data<Arc<Mutex<api_models::APPState>>>,
    body: web::Json<api_models::RequestBody>,
) -> impl Responder {
    let mut app_state = data.lock().unwrap();

    let query_tuple = (body.city_id, body.temperature_unit);

    if app_state.has_valid_cache_for(query_tuple) {
        let cached_response = app_state.get_cache_for(query_tuple).unwrap();
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
                if let Err(msg) = app_state.cache_response(response, query_tuple.1) {
                    log::warn!(
                        "Failed to created cache for ({}|{}) - {}",
                        query_tuple.0,
                        query_tuple.1,
                        msg
                    );
                }

                HttpResponse::Ok().json(api_models::RequestResponse::build_success(response))
            }
            Err(err) => {
                HttpResponse::Ok().json(api_models::RequestResponse::build_failure(err.to_string()))
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();

    let api_key: String = "c5c5aabf57fb5e923352a7cb40469df7".into();

    let app_state = api_models::APPState::build(api_key);

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
