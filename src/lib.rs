pub mod modbus_rtu;

use serde_derive::{Deserialize, Serialize};
use serde_json::json;

#[derive(clap::Parser)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(value_name = "FILE", required = true)]
    pub config: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Sensor {
    pub name: String,
    pub friendly_name: String,
    pub address: u16,
    pub accuracy: f64,
    pub unit: String,
    pub state_class: String,
    pub device_class: String,
}

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
        value: f64,
    ) -> Result<reqwest::Response, reqwest::Error> {
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

        match zero_decimal(value) {
            true => {
                // add state as i64
                post_data["state"] = (value as i64).into();

                client.json(&post_data).send().await
            }
            false => {
                // add state as f64
                post_data["state"] = value.into();

                client.json(&post_data).send().await
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    pub homeassistant: Homeassistant,
    pub modbus_rtu: modbus_rtu::Monitor,
}

// check if floating value can be removed
fn zero_decimal(float_value: f64) -> bool {
    let decimal = float_value.fract();

    decimal.abs() < std::f64::EPSILON
}
