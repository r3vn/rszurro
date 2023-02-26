use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use tokio_serial::SerialStream;
use tokio_modbus::prelude::*;
use tokio_serial::SerialPortBuilder;

#[derive(clap::Parser)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(value_name = "FILE", required = true )]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct SerialConfig {
    pub tty_path: String,
    pub baud_rate: u32,
    pub sleep_ms: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HomeassistantConfig {
    pub url: String,
    pub api_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Slave {
    pub name: String,
    pub address: u8,
    pub sensors: Vec<Sensor>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    pub homeassistant: HomeassistantConfig,
    pub serialport: SerialConfig,
    pub slaves: Vec<Slave>,
}

pub struct Connection {
    pub slave: Slave,
    pub ctx: tokio_modbus::client::Context,
    pub last_value_map: HashMap<u16, f64>,
}

impl Connection {
    // Connect to modbus slave
    pub async fn connect(
        slave: Slave,
        serial_conn: &SerialPortBuilder,
    ) -> Result<Self, Box<dyn Error>> {

        let ctx = rtu::connect_slave(SerialStream::open(serial_conn).unwrap(), Slave(slave.address))
            .await?;

        Ok(Self {
            slave,
            ctx,
            last_value_map: HashMap::new(),
        })
    }
}

pub async fn send_to_homeassistant(
        homeassistant: &HomeassistantConfig,
        slave: &Slave,
        sensor: &Sensor,
        value: f64
    ) -> Result<reqwest::Response, reqwest::Error> {

    // home assistant url
    let ha_url = format!(
        "{}/api/states/sensor.{}_{}",
        homeassistant.url, slave.name, sensor.name
    );

    // build json
    let post_data = json!({
        "state": value,
        "attributes": {
            "unit_of_measurement": sensor.unit,
            "device_class": sensor.device_class,
            "friendly_name": sensor.friendly_name,
            "state_class": sensor.state_class
        }
    });

    // post request
    let client = reqwest::Client::new();

    client.post(ha_url)
        .header("Content-type", "application/json")
        .header("Authorization", "Bearer ".to_owned() + &homeassistant.api_key)
        .json(&post_data)
        .send()
        .await
}
