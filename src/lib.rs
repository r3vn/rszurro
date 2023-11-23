pub mod endpoints;
pub mod watchers;
pub mod cache_manager;

pub use cache_manager::CacheManager;

use tokio::sync::mpsc;
use serde_derive::{Deserialize, Serialize};

#[derive(clap::Parser)]
pub struct Cli {
    // Sets a custom config file
    #[arg(value_name = "FILE", required = true)]
    pub config: String,

    // verbosity level 
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    // disable caching
    #[arg(long, action)]
    pub nocache: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,

    #[cfg(feature = "modbus-rtu")]
    pub modbus_rtu: watchers::ModbusRTU,

    #[cfg(feature = "lmsensors")]
    pub lm_sensors: watchers::LMSensors,

    #[cfg(feature = "sysinfo")]
    pub sysinfo: watchers::SysInfo,

    #[cfg(feature = "gpio")]
    pub gpio: watchers::Gpio,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Endpoint {
    pub name: String,
    pub enabled: bool,

    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub api_key: String,
}
impl Endpoint {
    pub async fn send(
        &self,
        update: SensorUpdate
    ) {
        // initialize endpoint
        let edp = match self.name.as_str() {
            "homeassistant" => endpoints::Homeassistant {
                url: self.url.clone(),
                api_key: self.api_key.clone(),
            },
            &_ => todo!(),
        };

        // send data
        tokio::spawn(async move{
            edp.send(update).await
        });
    }
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

#[derive(Clone)]
pub struct SensorUpdate {
    pub device_name: String, 
    pub sensor: Sensor,
    pub value: SensorValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorValue {
    IsBool(bool),
    IsF64(f64),
    IsString(String),
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

pub async fn update_sensor(
    tx: &mpsc::Sender<SensorUpdate>,
    device_name: &String,
    sensor: &Sensor,
    value: SensorValue,
) {
    // instantiating SensorUpdate
    let update = SensorUpdate {
        device_name: device_name.to_string(),
        sensor: sensor.clone(),
        value
    };

    // send sensor update to cache channel
    tx.send(update).await.unwrap();
}

pub fn update_sensor_sync(
    tx: &mpsc::Sender<SensorUpdate>,
    device_name: &String,
    sensor: &Sensor,
    value: SensorValue,
) {
    // instantiating SensorUpdate
    let update = SensorUpdate {
        device_name: device_name.to_string(),
        sensor: sensor.clone(),
        value
    };

    // send sensor update to cache channel
    tx.blocking_send(update).unwrap();
}

