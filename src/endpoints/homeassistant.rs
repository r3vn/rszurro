use crate::{Endpoint, SensorUpdate, SensorValue};
use log::{debug, error};

pub async fn send(endpoint: Endpoint, update: SensorUpdate) -> bool {
    // guess sensor type
    let prefix = match update.value {
        SensorValue::IsBool(_) => "binary_sensor",
        _ => "sensor",
    };

    // home assistant url
    let ha_url = format!(
        "{}/api/states/{}.{}_{}",
        &endpoint.url, prefix, &update.device_name, &update.sensor.name
    );

    // build client
    let client = reqwest::Client::new()
        .post(ha_url)
        .header("Content-type", "application/json")
        .header("Authorization", "Bearer ".to_owned() + &endpoint.api_key);

    // send sensor value
    match client.json(&update.get_json().await).send().await {
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
