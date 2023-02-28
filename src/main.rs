use tokio::time::{sleep, Duration};
use tokio_modbus::prelude::Reader;
use clap::Parser;

use rszurro::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // parse cli arguments
    let cli = Cli::parse();

    // read configuration file
    let configuration = {
        let configuration = std::fs::read_to_string(&cli.config)?;
        serde_json::from_str::<ConfigFile>(&configuration).unwrap()
    };

    // Make serial connection
    let builder = tokio_serial::new(
        configuration.serialport.tty_path,
        configuration.serialport.baud_rate);

    // establish connections with modbus slaves
    let mut connections = Vec::new();

    for slave in configuration.slaves {
        let connection = Connection::new(slave, &builder).await?;
        connections.push(connection);
    }

    loop {
        for conn in &mut connections {
            for sensor in &conn.slave.sensors {
                let sensor_value = {
                    let modbus_value = conn.ctx
                        .read_holding_registers(
                            sensor.address,
                            1)
                        .await;

                    match modbus_value {
                        Ok(rsp) => {
                            let float_value = f64::trunc(rsp
                                .iter()
                                .map(|&val| val as i64)
                                .sum::<i64>() as f64
                            * sensor.accuracy * 100.0) / 100.0;

                            if cli.verbose > 2 {
                                println!("slave: {} reg: {} - {}: {}{}",
                                    &conn.slave.name,
                                    &sensor.address,
                                    &sensor.name,
                                    &float_value,
                                    &sensor.unit);
                            }

                            float_value
                        },
                        Err(e) => {
                            if cli.verbose > 0 {
                                println!("slave: {} reg: {} - !!! error reading modbus register: {}.",
                                    &conn.slave.name,
                                    &sensor.address,
                                    &e);
                            }

                            continue;
                        }
                    }
                };

                // Check if the value changed since last loop
                if conn.last_value_map.get(&sensor.address) != Some(&sensor_value) {
                    if cli.verbose > 1 {
                        println!("slave: {} reg: {} - value changed sending to HA...",
                            &conn.slave.name,
                            &sensor.address);
                    }

                    // Send data to HA
                    let ha_rx = send_to_homeassistant(
                        &configuration.homeassistant,
                        &conn.slave,
                        &sensor,
                        sensor_value).await;

                    match ha_rx {
                        Err(e) =>
                            if cli.verbose > 0 {
                                println!("slave: {} reg: {} - error: {}, sleeping...",
                                    &conn.slave.name,
                                    &sensor.address,
                                    &e);

                            sleep(Duration::from_millis(configuration.serialport.sleep_ms * 2))
                                .await;
                            },

                        Ok(_) =>
                            if cli.verbose > 1 {
                                println!("slave: {} reg: {} - done.",
                                    &conn.slave.name,
                                    &sensor.address);
                            }
                    }

                    // Add sensor value on current_value_map, update value if any
                    conn.last_value_map.insert(sensor.address, sensor_value);
                }

                // prevent issues with serial
                sleep(Duration::from_millis(configuration.serialport.sleep_ms)).await;
            }
        }
    }
}
