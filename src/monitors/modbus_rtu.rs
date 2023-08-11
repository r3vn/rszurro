use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio_modbus::prelude::{rtu, Reader};
use tokio_serial::SerialStream;

use crate::{update_sensor, Endpoint, Sensor, SensorValue};

#[derive(Deserialize, Serialize, Debug)]
pub struct Slave {
    pub name: String,
    pub address: u8,
    pub sensors: Vec<Sensor>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SerialConfig {
    pub tty_path: String,
    pub baud_rate: u32,
    pub sleep_ms: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ModbusRTU {
    pub enabled: bool,
    pub serialport: SerialConfig,
    pub slaves: Vec<Slave>,
}

impl ModbusRTU {
    pub async fn run(
        &self,
        endpoints: Vec<Endpoint>,
        verbosity: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Make serial connection
        let builder = tokio_serial::new(&self.serialport.tty_path, self.serialport.baud_rate);

        // init last value map
        let mut last_value_map = HashMap::new();

        loop {
            for slave in &self.slaves {
                // Connect to modbus slave
                let mut ctx = rtu::connect_slave(
                    SerialStream::open(&builder).unwrap(),
                    tokio_modbus::slave::Slave(slave.address),
                )
                .await?;

                for sensor in &slave.sensors {
                    // Read sensor value from modbus
                    let sensor_value = {
                        let modbus_value = ctx.read_holding_registers(sensor.address, 1).await;

                        match modbus_value {
                            // Convert modbus register's value to float and truncate it at two digit
                            Ok(rsp) => {
                                f64::trunc(
                                    rsp.iter().map(|&val| val as i64).sum::<i64>() as f64
                                        * sensor.accuracy
                                        * 100.0,
                                ) / 100.0
                            }

                            // Modbus error
                            Err(e) => {
                                if verbosity > 0 {
                                    println!(
                                        "[modbus_rtu] slave: {} reg: {} - !!! error reading modbus register: {}.",
                                        &slave.name, &sensor.address, &e
                                    );
                                }
                                continue;
                            }
                        }
                    };

                    if verbosity > 2 {
                        println!(
                            "[modbus_rtu] slave: {} reg: {} - {}: {}{}",
                            &slave.name, &sensor.address, &sensor.name, &sensor_value, &sensor.unit
                        );
                    }

                    // Check if the value changed since last loop
                    if last_value_map.get(&sensor.address) != Some(&sensor_value) {
                        if verbosity > 1 {
                            println!(
                                "[modbus_rtu] slave: {} reg: {} - value changed sending to endpoints...",
                                &slave.name, &sensor.address
                            );
                        }

                        // Send data to HA
                        update_sensor(
                            &endpoints,
                            &slave.name,
                            sensor,
                            SensorValue::IsF64(sensor_value),
                        )
                        .await;

                        // Add sensor value on current_value_map, update value if any
                        last_value_map.insert(sensor.address, sensor_value);
                    }

                    // prevent issues with serial
                    sleep(Duration::from_millis(self.serialport.sleep_ms)).await;
                }

                // Disconnect the client
                let _cls = ctx.disconnect().await;
            }
        }
    }
}
