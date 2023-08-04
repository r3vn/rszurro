use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::Sensor;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Homeassistant {
    pub url: String,
    pub api_key: String,
}

impl Homeassistant {
    pub async fn send(
        &self, 
        device_name: &String, 
        sensor: &Sensor, 
        value: f64
    ) -> bool {
        // home assistant url
        let ha_url = format!(
            "{}/api/states/sensor.{}_{}",
            &self.url, device_name, sensor.name
        );

        // build json
        let mut post_data = json!({
            "attributes": {
                "unit_of_measurement": sensor.unit,
                "device_class": sensor.device_class,
                "friendly_name": sensor.friendly_name,
                "state_class": sensor.state_class
            }
        });

        let client = reqwest::Client::new()
            .post(ha_url)
            .header("Content-type", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.api_key);

        post_data["state"] = match zero_decimal(value) {
            true => (value as i64).into(), // add state as i64
            false => value.into(),         // add state as f64
        };

        match client.json(&post_data).send().await {
            Err(e) => {
                println!("[homeassistant] {}", e);
                false
            }
            Ok(_) => true,
        }
    }

    pub fn send_sync(
        &self, 
        device_name: &String, 
        sensor: &Sensor, 
        value: f64
    ) -> bool {
        // home assistant url
        let ha_url = format!(
            "{}/api/states/sensor.{}_{}",
            &self.url, device_name, sensor.name
        );

        // build json
        let mut post_data = json!({
            "attributes": {
                "unit_of_measurement": sensor.unit,
                "device_class": sensor.device_class,
                "friendly_name": sensor.friendly_name,
                "state_class": sensor.state_class
            }
        });

        let client = reqwest::blocking::Client::new()
            .post(ha_url)
            .header("Content-type", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.api_key);

        post_data["state"] = match zero_decimal(value) {
            true => (value as i64).into(), // add state as i64
            false => value.into(),         // add state as f64
        };

        match client.json(&post_data).send() {
            Err(e) => {
                println!("[homeassistant] {}", e);
                false
            }
            Ok(_) => true,
        }
    }
}

fn zero_decimal(float_value: f64) -> bool {
    // check if floating value can be removed
    let decimal = float_value.fract();

    decimal.abs() < std::f64::EPSILON
}
