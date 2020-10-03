use actix_web::{middleware::Logger, web};
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use log;
use std::sync::{Arc, Mutex};

mod api_models;
mod weather_api;

#[post("/weather")]
async fn current_weather_route(
    data: web::Data<Arc<Mutex<api_models::APPState>>>,
    body: web::Json<api_models::RequestBody>,
) -> impl Responder {

    let mut app_state = data.lock().unwrap();

    let city_id = body.city_id;
    let temperature_fmt = body.temp_format;

    if app_state.has_valid_cache_for(city_id) {
        return HttpResponse::Ok().json(app_state.get_cache_for(city_id).unwrap());
    } else {
        let request_result =
            weather_api::get_weather_data_for(city_id, temperature_fmt, &app_state.api_key);

        match request_result.await {
            Some(response) => {
                if let Err(msg) = app_state.cache_response(response) {
                    log::warn!("Failed to created cache for {} - {}", city_id, msg);
                }

                HttpResponse::Ok().json(&response)
            }
            None => HttpResponse::Ok().json("b"),
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
