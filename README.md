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

## Running the project

You need to set up to environment variables before running the project:

- `CITY_DATABASE_PATH`: Path for the `cities_db.json` file, you can find the file with the project.
- `OPENWEATHER_API_KEY`: Here you need to place your OpenWeatherMap API key. A key can be obtained by creating an account at [OpenWeatherMap](https://openweathermap.org/api/)

Also if you wish to run the server in production mode which will simply log less output in the terminal, set the variable `WEATHER_API_SERVER_PROD` in you environment.

Once the previous steps are complete. Simply execute the command `cargo run --release` command or alternativelly `cargo intall --path .` and `weather-retrieve`.
