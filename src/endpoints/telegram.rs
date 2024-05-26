use crate::{Endpoint, SensorUpdate, SensorValue};
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
    let message_body = match &update.last_value {
        SensorValue::None => format!(
            "*{}.{}*: {}{}",
            &update.device_name,
            &update.sensor.name,
            json_data["state"].to_string().replace('"', ""),
            json_data["attributes"]["unit_of_measurement"]
                .as_str()
                .expect("Value is a str")
        ),

        _ => format!(
            "*{}.{}*: {}{} *»* {}{}",
            &update.device_name,
            &update.sensor.name,
            json_data["attributes"]["previous_state"]
                .to_string()
                .replace('"', ""),
            json_data["attributes"]["unit_of_measurement"]
                .as_str()
                .expect("Value is a str"),
            json_data["state"].to_string().replace('"', ""),
            json_data["attributes"]["unit_of_measurement"]
                .as_str()
                .expect("Value is a str")
        ),
    };

    let post_data = json!({
        "chat_id"    : &endpoint.chat_id,
        "parse_mode" : "markdown",
        "text"       : &message_body
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
