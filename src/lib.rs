pub mod cache_manager;
pub mod endpoints;
pub mod watchers;

pub use cache_manager::CacheManager;

use log::error;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc;

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

#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,
    pub watchers: Vec<Watcher>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Endpoint {
    pub endpoint: String,
    pub name: String,

    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub port: u16,
}
impl Endpoint {
    pub async fn run(&self, update: SensorUpdate) -> tokio::task::JoinHandle<()> {
        // initialize endpoint
        let endpoint = self.clone();
        tokio::spawn(async move {
            match endpoint.endpoint.as_str() {
                #[cfg(feature = "endpoint_homeassistant")]
                "homeassistant" => endpoints::homeassistant::send(endpoint, update).await,

                _ => {
                    error!("unsupported endpoint: {}", &endpoint.endpoint);
                    false
                }
            };
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Watcher {
    pub name: String, // replace device_name !!!
    pub watcher: String,

    #[serde(default)]
    pub sensors: Vec<Sensor>,

    #[serde(default)]
    pub slaves: Vec<Slave>,

    #[serde(default = "empty_string")]
    pub chip: String,

    #[serde(default = "empty_string")]
    pub path: String,

    #[serde(default)]
    pub baud_rate: u32,

    #[serde(default)]
    pub sleep_ms: u64,

    #[serde(default = "empty_string")]
    pub temperature_unit: String,
}
impl Watcher {
    pub async fn run(&self, tx: mpsc::Sender<SensorUpdate>) -> tokio::task::JoinHandle<()> {
        // run a watcher
        let watcher = self.clone();

        match watcher.watcher.as_str() {
            #[cfg(feature = "gpio")]
            "gpio" => tokio::spawn(async move { watchers::gpio::run(watcher, tx).await.unwrap() }),

            #[cfg(feature = "lmsensors")]
            "lm_sensors" => {
                tokio::task::spawn_blocking(move || watchers::lm_sensors::run(watcher, tx).unwrap())
            }

            #[cfg(feature = "modbus-rtu")]
            "modbus_rtu" => {
                tokio::spawn(async move { watchers::modbus_rtu::run(watcher, tx).await.unwrap() })
            }

            #[cfg(feature = "sysinfo")]
            "sysinfo" => {
                tokio::spawn(async move { watchers::sysinfo::run(watcher, tx).await.unwrap() })
            }
            &_ => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Slave {
    #[serde(default)]
    pub sensors: Vec<Sensor>,

    #[serde(default)]
    pub address: u8,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Sensor {
    pub name: String,

    #[serde(default = "empty_string")]
    pub friendly_name: String,

    #[serde(default = "sensor_default_bool")]
    pub is_bool: bool,

    #[serde(default)]
    pub address: u16,

    #[serde(default = "sensor_default_accuracy")]
    pub accuracy: f64,

    #[serde(default = "empty_string")]
    pub unit: String,

    #[serde(default = "empty_string")]
    pub state_class: String,

    #[serde(default = "empty_string")]
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

fn empty_string() -> String {
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
        value,
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
        value,
    };

    // send sensor update to cache channel
    tx.blocking_send(update).unwrap();
}
