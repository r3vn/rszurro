use tokio_serial::{SerialStream, SerialPortBuilder};
use tokio_modbus::prelude::rtu;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

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
    pub async fn new(
        slave: Slave,
        serial_conn: &SerialPortBuilder,
    ) -> Result<Self, Box<dyn Error>> {

        let ctx = rtu::connect_slave(
                SerialStream::open(serial_conn).unwrap(),
                tokio_modbus::slave::Slave(slave.address))
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
        .header("Authorization", "Bearer ".to_owned() + &homeassistant.api_key);

    match zero_decimal(value){
        true => {
            // add state as i64
            post_data["state"] = (value as i64).into();

            client
                .json(&post_data)
                .send()
                .await
        },
        false => {
            // add state as f64
            post_data["state"] = value.into();

            client
                .json(&post_data)
                .send()
                .await
        }
    }
}

// check if floating value can be removed
fn zero_decimal(float_value: f64) -> bool {
    let decimal = float_value.fract();

    decimal.abs() < std::f64::EPSILON
}

