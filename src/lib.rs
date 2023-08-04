pub mod endpoints;
pub mod monitors;

use serde_derive::{Deserialize, Serialize};

#[derive(clap::Parser)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(value_name = "FILE", required = true)]
    pub config: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,
    pub modbus_rtu: monitors::ModbusRTU,
    pub lm_sensors: monitors::LMSensors,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Endpoint {
    pub name: String,
    pub enabled: bool,

    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub api_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Sensor {
    pub name: String,

    #[serde(default)]
    pub friendly_name: String,

    #[serde(default)]
    pub address: u16,

    #[serde(default)]
    pub accuracy: f64,

    #[serde(default)]
    pub unit: String,

    #[serde(default)]
    pub state_class: String,

    #[serde(default)]
    pub device_class: String,
}

pub async fn update_sensor(
    endpoints: &Vec<Endpoint>,
    device_name: &String,
    sensor: &Sensor,
    value: f64
) {
    // send sensor's data to endpoints async
    for endpoint in endpoints {

        // endpoint is disabled
        if !endpoint.enabled { continue }

        // initialize endpoints
        let edp = match endpoint.name.as_str() {
            "homeassistant" => endpoints::Homeassistant {
                    url: endpoint.url.clone(),
                    api_key: endpoint.api_key.clone()
            },
            "_" => continue,
            &_ => todo!()
        };
    
        // send data
        edp.send(
            device_name,
            sensor,
            value).await;
    }
}
pub fn update_sensor_sync(
    endpoints: &Vec<Endpoint>, 
    device_name: &String, 
    sensor: &Sensor, 
    value: f64
) {
    // send sensor's data to endpoints sync
    for endpoint in endpoints {

        // endpoint is disabled
        if !endpoint.enabled { continue }

        // initialize endpoint
        let edp = match endpoint.name.as_str() {
            "homeassistant" => endpoints::Homeassistant {
                    url: endpoint.url.clone(),
                    api_key: endpoint.api_key.clone()
            },
            "_" => continue,
            &_ => todo!()
        };

        // send data
        edp.send_sync(
            device_name,
            sensor,
            value);
    }
}
