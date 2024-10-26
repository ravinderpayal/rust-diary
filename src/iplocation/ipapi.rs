//service/ipapi.rs

use reqwest;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde_json::Value;

pub async fn get_ip_location() -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://ipapi.co/json/");
    let custom_user_agent = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, custom_user_agent.parse().unwrap());
    let client = reqwest::Client::new();
    let response: Value = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map(|op| op.json())?
        .await?;

    // println!("JSON: {}", response.to_string());
    let city = response["city"].as_str().unwrap_or("");
    let region = response["region"].as_str().unwrap_or("");
    let country = response["country"].as_str().unwrap_or("");

    let mut current_location = if city.len() > 0 && (region.len() > 0 || country.len() > 0) {
        format!("{},", city)
    } else {
        city.to_string()
    };

    if region.len() > 0 && country.len() > 0 {
        current_location = format!("{}{},", current_location, region);
    }
    current_location = format!("{}{}", current_location, country);
    Ok(current_location)
}
