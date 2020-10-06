# weather-retrieve OpenWeatherMap Rust API server

This code implements something similar to a "proxy" for certain [OpenWeatherMap](https://openweathermap.org/api/) API endpoints, 
specifically for the [One Call API](https://openweathermap.org/api/one-call-api) endpoint.

## Provided endpoints

The server has two endpoints: `/weather` used to query current weather conditions and `/forecast` to query the hourly forecast for the next 48 hours.

Both endpoints are `HTTP GET` endpoints, with the parameters supply as a JSON in the body of the query, like the example below:

```sh
curl -d '{"city_query":"Madrid,ES", "units":"C"}' -H "Content-Type: application/json" -X GET http://localhost:8080/forecast
```

As you can see the query has two parameters.

### Query parameters

The first parameter is the `city_query` that consists of a city name that begins with a capitalized city name, followed by an `,` 
character and an **ISO 3166-1 alfa-2** country code.

The second paramater is the `units` parameter, which helps indicate the temperature and other weather units, the valid values are the following:
- "C": "Metric units"
- "F": "Imperial units"
- "K": "Standard units, (temperature in Kelvin)"

## Running the project

You need to set up to environment variables before running the project:

- `CITY_DATABASE_PATH`: Path for the `cities_db.json` file, you can find the file with the project.
- `OPENWEATHER_API_KEY`: Here you need to place your OpenWeatherMap API key. A key can be obtained by creating an account at [OpenWeatherMap](https://openweathermap.org/api/)

Also if you wish to run the server in production mode which will simply log less output in the terminal, set the variable `WEATHER_API_SERVER_PROD` in you environment.

Once the previous steps are complete. Simply execute the command `cargo run --release` command or alternativelly `cargo intall --path .` and then `weather-retrieve`.

## Running the tests

To run the tests simply run `cargo test` while inside the project directory.

## Project architeture

The project is divided into multiple modules that encompass different functionalities:

* The `main` module contains the ActiX Web Server and the endpoint definitions.
* The `app_state` module contains the `AppState` struct which is the container for the shared state between the different ActiX workers.
    * It also contains the `CachedElement` struct which is the generic base for the request caching mechanism.
* In the `weather_api` module we have the `APIClient` struct which is the one tasked with query the OpenWeatherMap endpoint to retrieve the data requested in one of the application's own endpoints.
* The `utils` module contains some methods which are used during initialization of the ActiX web server.
* The `models` folder contains the various structs that are serialized/deserialized through the application.
    * The `api` submodule contains the structs that model the API reponses from the OpenWeatherMap calls perfomed by the `APIClient` struct.
    * The `request` submodule has the various struct used in the ActiX web server endpoint parameters both the Requests and the Reponses
    * Finally the `state` submodule contains the structs used in the `AppState` class, for example the API Cache dictionary keys.

## Project dependencies

The power this project a certain number of crates have been used, they are the following:

* [actix-web](https://crates.io/crates/actix-web) - Powering the Web server
* [serde](https://crates.io/crates/serde) - For serializing/deserializing the multitude of models
* [serde_with](https://crates.io/crates/serde_with) - For additional serialization/deserialization attributes
* [serde_json](https://crates.io/crates/serde_json) - For serializing/deserializing JSON requests and responses
* [reqwest](https://crates.io/crates/reqwest) - For query the OpenWeatherMap API
* [env_logger](https://crates.io/crates/env_logger) - For ActiX logging
* [log](https://crates.io/crates/log) - For the app logging

Also for async testing:

* [actix-rt](https://crates.io/crates/actix-rt) - To be able to run `async fn` in tests

## Deployment in production

Even though the application could be run in a production environment, the ideal approach would be to build a Docker image containig the 
optimized binary and the necessary environment variables as secrets, and then deploy that image either in **AWS ECS**, **Google Compute Engine** or **Azure Container**, with the option
to also use a Kubernetes cluster if scalability is a concern.
