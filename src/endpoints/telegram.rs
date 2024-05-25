use crate::{Endpoint, SensorUpdate};
use log::{debug, error};
use serde_json::json;

pub async fn send(endpoint: Endpoint, update: SensorUpdate) -> bool {
    // telegram api url
    let ha_url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        &endpoint.api_key //, &endpoint.chat_id
    );

    // build client
    let client = reqwest::Client::new()
        .post(ha_url)
        .header("Content-type", "application/json");

    let json_data = &update.get_json().await;
    let post_data = json!({
        "chat_id"    : &endpoint.chat_id,
        "parse_mode" : "markdown",
        "text"       : format!(
            "*{}.{}*: {}{}",
            &update.device_name,
            &update.sensor.name,
            json_data["state"].to_string().replace('"',""),
            json_data["attributes"]["unit_of_measurement"].as_str().expect("Value is a str")
        )
    });

    // send sensor value
    match client.json(&post_data).send().await {
        Err(e) => {
            error!("{}: {}", &update.sensor.name, e);
            false
        }
        Ok(_) => {
            debug!("{}: updated successfully.", &update.sensor.name);
            true
        }
    }
}
