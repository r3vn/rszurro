use tokio_serial::SerialStream;
use tokio_modbus::prelude::*;
use tokio::time::{sleep, Duration};
use clap::Parser;
use std::collections::HashMap;

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

    let mut connections: Vec<tokio_modbus::client::Context> = vec![];
    let mut previous_value_map: Vec<HashMap<usize, f64>> = vec![];

    for index in 0..configuration.slaves.len() {
        // connect to slave
        let ctx = rtu::connect_slave(
            SerialStream::open(&builder).unwrap(),
            Slave(configuration.slaves[index].address))
            .await?;

        // make an hashmap for each slave
        previous_value_map.push(HashMap::new());

        // add connection into a list
        connections.push(ctx);
    }

    loop {
        for slave_i in 0..configuration.slaves.len() {

            let mut current_value_map: HashMap<usize, f64> = HashMap::new();

            for sensor_i in 0..configuration.slaves[slave_i].sensors.len() {

                // Read modbus register value and convert u16 response to f64
                let rsp = {
                    let rsp = connections[slave_i]
                        .read_holding_registers(
                            configuration.slaves[slave_i].sensors[sensor_i].address,
                            1)
                        .await?
                        .iter()
                        .map(|&val| val as i64)
                        .sum::<i64>() as f64;

                    // multiply by accuracy and truncate float decimals to 2
                    f64::trunc(rsp * configuration.slaves[slave_i].sensors[sensor_i].accuracy * 100.0) / 100.0
                };

                if cli.verbose > 0 {
                    print!("slave: {} reg: {} - {}: {}{}",
                        &configuration.slaves[slave_i].name,
                        &configuration.slaves[slave_i].sensors[sensor_i].address,
                        &configuration.slaves[slave_i].sensors[sensor_i].name,
                        &rsp,
                        &configuration.slaves[slave_i].sensors[sensor_i].unit);
                }

                // Check if the value changed since last loop
                if previous_value_map[slave_i].get(&sensor_i) != Some(&rsp) {
                    if cli.verbose > 1 {
                        print!(" - sending value to HA...");
                    }

                    // Send data to HA
                    send_to_homeassistant(
                        &configuration.homeassistant,
                        &configuration.slaves[slave_i],
                        &configuration.slaves[slave_i].sensors[sensor_i],
                        rsp).await?;
                }

                if cli.verbose > 0 {
                    println!("");
                }

                // Add sensor value on current_value_map
                current_value_map.insert(sensor_i, rsp);

                // prevent issues with serial
                sleep(Duration::from_millis(configuration.serialport.sleep_ms)).await;
            }

            // update previous values, this consume current_value
            previous_value_map[slave_i].clear();
            previous_value_map[slave_i].extend(current_value_map.into_iter());
        }
    }
}
