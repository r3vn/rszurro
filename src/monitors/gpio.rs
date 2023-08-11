use crate::{update_sensor, Endpoint, Sensor, SensorValue};
use serde_derive::{Deserialize, Serialize};
use tokio_gpiod::{Chip, Edge, EdgeDetect, Options};

#[derive(Deserialize, Serialize, Debug)]
pub struct Gpio {
    pub enabled: bool,
    pub gpio_chip: String,
    pub sensors: Vec<Sensor>,
}

impl Gpio {
    pub async fn run(
        &self,
        endpoints: Vec<Endpoint>,
        verbosity: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut handles = vec![];

        for sensor in self.sensors.clone() {
            let chip_name = self.gpio_chip.clone();
            let ep = endpoints.clone();

            handles.push(tokio::spawn(async move {
                // spawn an handle for each sensor
                let chip = Chip::new(&chip_name).await.unwrap();

                let opts = Options::input([23]) // configure lines offsets
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
                    update_sensor(&ep, &chip_name, &sensor, SensorValue::IsBool(sensor_value))
                        .await;

                    if verbosity > 2 {
                        println!(
                            "[gpio] {} - {} event: {:?}",
                            &chip_name, &sensor.address, event
                        );
                    }
                }
            }));
        }

        // wait sensors tasks
        futures::future::join_all(handles).await;
        Ok(())
    }
}
