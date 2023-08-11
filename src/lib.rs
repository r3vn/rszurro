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

    #[cfg(feature = "modbus-rtu")]
    pub modbus_rtu: monitors::ModbusRTU,

    #[cfg(feature = "lmsensors")]
    pub lm_sensors: monitors::LMSensors,

    #[cfg(feature = "sysinfo")]
    pub sysinfo: monitors::SysInfo,

    #[cfg(feature = "gpio")]
    pub gpio: monitors::Gpio,
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

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Sensor {
    pub name: String,

    #[serde(default = "sensor_empty_string")]
    pub friendly_name: String,

    #[serde(default = "sensor_default_bool")]
    pub is_bool: bool,

    #[serde(default)]
    pub address: u16,

    #[serde(default = "sensor_default_accuracy")]
    pub accuracy: f64,

    #[serde(default = "sensor_empty_string")]
    pub unit: String,

    #[serde(default = "sensor_empty_string")]
    pub state_class: String,

    #[serde(default = "sensor_empty_string")]
    pub device_class: String,
}

fn sensor_empty_string() -> String {
    "".to_string()
}

fn sensor_default_bool() -> bool {
    false
}

fn sensor_default_accuracy() -> f64 {
    1.0
}

#[derive(Clone)]
pub enum SensorValue {
    IsBool(bool),
    IsF64(f64),
    IsString(String),
}

pub async fn update_sensor(
    endpoints: &Vec<Endpoint>,
    device_name: &String,
    sensor: &Sensor,
    value: SensorValue,
) {
    // send sensor's data to endpoints async
    for endpoint in endpoints {
        // endpoint is disabled
        if !endpoint.enabled {
            continue;
        }

        // initialize endpoints
        let edp = match endpoint.name.as_str() {
            "homeassistant" => endpoints::Homeassistant {
                url: endpoint.url.clone(),
                api_key: endpoint.api_key.clone(),
            },
            "_" => continue,
            &_ => todo!(),
        };

        // send data
        edp.send(device_name, sensor, value.clone()).await;
    }
}

pub fn update_sensor_sync(
    endpoints: &Vec<Endpoint>,
    device_name: &String,
    sensor: &Sensor,
    value: SensorValue,
) {
    // send sensor's data to endpoints sync
    for endpoint in endpoints {
        // endpoint is disabled
        if !endpoint.enabled {
            continue;
        }

        // initialize endpoint
        let edp = match endpoint.name.as_str() {
            "homeassistant" => endpoints::Homeassistant {
                url: endpoint.url.clone(),
                api_key: endpoint.api_key.clone(),
            },
            "_" => continue,
            &_ => todo!(),
        };

        // send data
        edp.send_sync(device_name, sensor, value.clone());
    }
}
