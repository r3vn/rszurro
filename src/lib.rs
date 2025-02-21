pub mod cache_manager;
pub mod endpoints;
pub mod watchers;

pub use cache_manager::CacheManager;

use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::mpsc, sync::Mutex};

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
    pub platform: String,
    pub name: String,

    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub host: String,

    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub password: String,

    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub chat_id: String,

    #[serde(default)]
    pub port: u16,

    #[serde(default)]
    pub raw: bool,

    #[serde(default)]
    pub keepalive: u64,

    #[serde(default)]
    pub prefix: String,

    #[serde(default)]
    pub ca: String,

    #[serde(default)]
    pub client_crt: String,

    #[serde(default)]
    pub client_key: String,
}
impl Endpoint {
    pub async fn get_client(&self, state: Arc<Mutex<bool>>) -> Client {
        let endpoint = self.clone();

        match endpoint.platform.as_str() {
            #[cfg(feature = "mqtt")]
            "mqtt" => endpoints::mqtt::get_client(endpoint, state).await,

            _ => Client::None,
        }
    }

    pub async fn run(&self, update: SensorUpdate, client: Client) -> tokio::task::JoinHandle<()> {
        // initialize endpoint
        let endpoint = self.clone();

        tokio::spawn(async move {
            match endpoint.platform.as_str() {
                #[cfg(feature = "telegram")]
                "telegram" => endpoints::telegram::send(endpoint, update).await,

                #[cfg(feature = "homeassistant")]
                "homeassistant" => endpoints::homeassistant::send(endpoint, update).await,

                #[cfg(feature = "mqtt")]
                "mqtt" => endpoints::mqtt::send(endpoint, update, client).await,

                _ => {
                    error!("unsupported endpoint platform: {}", &endpoint.platform);
                    false
                }
            };
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Watcher {
    pub name: String,
    pub platform: String,

    #[serde(default)]
    pub sensors: Vec<Sensor>,

    #[serde(default)]
    pub slaves: Vec<Slave>,

    #[serde(default)]
    pub host: String,

    #[serde(default)]
    pub chip: String,

    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub baud_rate: u32,

    #[serde(default)]
    pub scan_interval: u64,

    #[serde(default)]
    pub timeout: u64,

    #[serde(default)]
    pub count: u64,

    #[serde(default)]
    pub temperature_unit: String,
}
impl Watcher {
    pub async fn run(&self, tx: mpsc::Sender<SensorUpdate>) -> tokio::task::JoinHandle<()> {
        // run a watcher
        let watcher = self.clone();

        match watcher.platform.as_str() {
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

            #[cfg(feature = "icmp")]
            "icmp" => tokio::spawn(async move { watchers::icmp::run(watcher, tx).await.unwrap() }),

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

    #[serde(default)]
    pub friendly_name: String,

    #[serde(default)]
    pub debounce_delay: u64,

    #[serde(default)]
    pub address: u16,

    #[serde(default = "sensor_default_accuracy")]
    pub accuracy: f64,

    #[serde(default)]
    pub unit: String,

    #[serde(default)]
    pub state_class: String,

    #[serde(default)]
    pub device_class: String,
}
impl Sensor {
    pub async fn new(name: String, friendly_name: String) -> Self {
        // instantiate a sensor
        Self {
            name,
            friendly_name,
            address: 0,
            accuracy: 0.0,
            unit: "".to_string(),
            state_class: "".to_string(),
            device_class: "".to_string(),
            debounce_delay: 0,
        }
    }
}

#[derive(Clone)]
pub struct SensorUpdate {
    pub device_name: String,
    pub sensor: Sensor,
    pub value: SensorValue,
    pub last_value: SensorValue,
}
impl SensorUpdate {
    pub async fn get_json(&self) -> serde_json::Value {
        // build json
        let mut data = json!({
            "attributes": {
                "unit_of_measurement": self.sensor.unit,
                "device_class": self.sensor.device_class,
                "friendly_name": self.sensor.friendly_name,
                "state_class": self.sensor.state_class,
            }
        });

        data["state"] = match self.value.clone() {
            SensorValue::IsBool(value) => serde_json::Value::String(if value {
                "on".to_string()
            } else {
                "off".to_string()
            }),

            SensorValue::IsF64(value) => match self.zero_decimal(value).await {
                true => (value as i64).into(), // add state as i64
                false => value.into(),         // add state as f64
            },

            SensorValue::IsString(value) => serde_json::Value::String(value),
            SensorValue::None => "off".into(),
        };

        data["attributes"]["previous_state"] = match self.last_value.clone() {
            SensorValue::IsBool(value) => serde_json::Value::String(if value {
                "on".to_string()
            } else {
                "off".to_string()
            }),

            SensorValue::IsF64(value) => match self.zero_decimal(value).await {
                true => (value as i64).into(), // add state as i64
                false => value.into(),         // add state as f64
            },

            SensorValue::IsString(value) => serde_json::Value::String(value),
            SensorValue::None => "off".into(),
        };

        data
    }

    async fn zero_decimal(&self, float_value: f64) -> bool {
        let decimal = float_value.fract();

        decimal.abs() < std::f64::EPSILON
    }
}

pub struct EndpointConnection {
    client: Client,
    state: Arc<Mutex<bool>>,
}

#[derive(Clone, Default)]
pub enum Client {
    #[default]
    None,

    #[cfg(feature = "mqtt")]
    MqttClient(rumqttc::AsyncClient),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorValue {
    IsBool(bool),
    IsF64(f64),
    IsString(String),
    None,
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
        last_value: SensorValue::None,
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
        last_value: SensorValue::None,
    };

    // send sensor update to cache channel
    tx.blocking_send(update).unwrap();
}

pub async fn read_file(filename: &String) -> Vec<u8> {
    // read a file as bytes
    let mut f = tokio::fs::File::open(&filename)
        .await
        .expect("no file found");

    let metadata = tokio::fs::metadata(&filename)
        .await
        .expect("unable to read file metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).await.expect("buffer overflow");

    buffer
}
