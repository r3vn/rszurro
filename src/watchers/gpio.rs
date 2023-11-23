use crate::{update_sensor, Sensor, SensorValue, SensorUpdate};
use serde_derive::{Deserialize, Serialize};
use tokio_gpiod::{Chip, Edge, EdgeDetect, Options};
use tokio::sync::mpsc;
use log::trace;

#[derive(Deserialize, Serialize, Debug)]
pub struct Gpio {
    pub enabled: bool,
    pub gpio_chip: String,
    pub sensors: Vec<Sensor>,
}

impl Gpio {
    pub async fn run(
        &self,
        tx: mpsc::Sender<SensorUpdate>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut handles = vec![];

        for sensor in self.sensors.clone() {
            let chip_name = self.gpio_chip.clone();
            let sensor_address = u32::from(sensor.address);
            let tx2 = tx.clone();

            handles.push(tokio::spawn(async move {
                // spawn an handle for each sensor
                let chip = Chip::new(&chip_name).await.unwrap();

                let opts = Options::input([sensor_address]) // configure lines offsets
                    .edge(EdgeDetect::Both) // configure edges to detect
                    .consumer("my-inputs"); // optionally set consumer string

                let mut inputs = chip.request_lines(opts).await.unwrap();

                loop {
                    // wait for gpio events
                    let event = inputs.read_event().await.unwrap();
                    let sensor_value = match event.edge {
                        Edge::Rising => true,
                        Edge::Falling => false,
                    };

                    // send value to endpoints
                    update_sensor(&tx2, &chip_name, &sensor, SensorValue::IsBool(sensor_value))
                        .await;

                    trace!("{} {} event: {:?}", &chip_name, &sensor.address, event);
                }
            }));
        }

        // wait sensors tasks
        futures::future::join_all(handles).await;
        Ok(())
    }
}
