use log::{error, trace};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_modbus::prelude::{rtu, Reader};
use tokio_serial::SerialStream;

use crate::{update_sensor, Sensor, SensorUpdate, SensorValue};

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
        tx: mpsc::Sender<SensorUpdate>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Make serial connection
        let builder = tokio_serial::new(&self.serialport.tty_path, self.serialport.baud_rate);

        loop {
            for slave in &self.slaves {
                // Connect to modbus slave
                let mut ctx = rtu::attach_slave(
                    SerialStream::open(&builder).unwrap(),
                    tokio_modbus::slave::Slave(slave.address),
                );

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
                                error!(
                                    "{} {} error reading modbus register: {}",
                                    &slave.name, &sensor.name, &e
                                );
                                continue;
                            }
                        }
                    };
                    trace!(
                        "{} {} => {}{}",
                        &slave.name,
                        &sensor.name,
                        &sensor_value,
                        &sensor.unit
                    );

                    // Send data to HA
                    update_sensor(&tx, &slave.name, sensor, SensorValue::IsF64(sensor_value)).await;

                    // prevent issues with serial
                    sleep(Duration::from_millis(self.serialport.sleep_ms)).await;
                }

                // Disconnect the client
                let _cls = ctx.disconnect().await;
            }
        }
    }
}
