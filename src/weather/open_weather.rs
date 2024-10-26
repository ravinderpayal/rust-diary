//service/open_weather.rs
//
use reqwest;
use serde_json::Value;

pub struct OpenWeatherService {
    city: String,
    api_key: String,
}

impl OpenWeatherService {
    pub fn new(city: &str, api_key: &str) -> Self {
        OpenWeatherService {
            city: city.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn get_weather(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "http://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            self.city, self.api_key
        );

        let response: Value = reqwest::blocking::get(&url)?.json()?;

        let temp = response["main"]["temp"].as_f64().unwrap_or(0.0);
        let description = response["weather"][0]["description"].as_str().unwrap_or("N/A");

        Ok(format!("Temperature: {:.1}°C, Conditions: {}", temp, description))
    }
}
