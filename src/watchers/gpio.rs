use crate::{update_sensor, SensorUpdate, SensorValue, Watcher};
use log::{debug, trace};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_gpiod::{Chip, Edge, EdgeDetect, Options};

pub async fn run(
    watcher: Watcher,
    tx: mpsc::Sender<SensorUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];

    for sensor in watcher.sensors.clone() {
        let chip_name = watcher.chip.clone();
        let sensor_address = u32::from(sensor.address);
        let tx2 = tx.clone();

        handles.push(tokio::spawn(async move {
            // spawn an handle for each sensor
            let chip = Chip::new(&chip_name).await.unwrap();

            let opts = Options::input([sensor_address]) // configure lines offsets
                .edge(EdgeDetect::Both) // configure edges to detect
                .consumer(&sensor.name); // optionally set consumer string

            let mut inputs = chip.request_lines(opts).await.unwrap();

            loop {
                // wait for gpio events
                let event = inputs.read_event().await.unwrap();
                let sensor_value = match event.edge {
                    Edge::Rising => true,
                    Edge::Falling => false,
                };

                if sensor.debounce_delay > 0 {
                    // Simple debounce mechanism using time-based sleep
                    sleep(Duration::from_millis(sensor.debounce_delay)).await;

                    // Read the state again after debounce
                    let debounced_state = inputs.get_values(sensor.address).await.unwrap();

                    // Check if the state is stable after debounce
                    if (debounced_state != 0) != sensor_value {
                        // bounce detected, ignore event
                        debug!(
                            "debounce: ignored false positive from {} gpio {}",
                            &chip_name, &sensor.address
                        );
                        continue;
                    }
                }

                trace!("{} {} event: {:?}", &chip_name, &sensor.address, event);

                // Send value to endpoints only if the state is stable
                update_sensor(&tx2, &chip_name, &sensor, SensorValue::IsBool(sensor_value)).await;
            }
        }));
    }

    // wait sensors tasks
    futures::future::join_all(handles).await;
    Ok(())
}
