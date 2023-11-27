use log::{debug, info, trace};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{Endpoint, SensorUpdate};

pub struct CacheManager {
    pub enabled: bool,
    pub endpoints: Vec<Endpoint>,
}
impl CacheManager {
    pub async fn run(&self, mut rx: mpsc::Receiver<SensorUpdate>) {
        // init cache
        let mut cache = HashMap::new();

        // wait for senders
        loop {
            match rx.recv().await {
                Some(update) => {
                    trace!(
                        "{} {}: {:?} received.",
                        &update.device_name,
                        &update.sensor.name,
                        &update.value
                    );

                    // get cached value for this sensor
                    let last_value = cache.get(&update.sensor.name);

                    // check if value changed from the cached one
                    if last_value != Some(&update.value) {
                        for endpoint in &self.endpoints {
                            // clone update
                            let update1 = update.clone();
                            //let endpoint1 = endpoint.clone();

                            // send data to endpoint
                            endpoint.run(update1).await;
                            info!(
                                "{} {}: {:?} => {}",
                                &update.device_name,
                                &update.sensor.name,
                                &update.value,
                                &endpoint.name
                            );
                        }

                        // check if caching is enabled
                        if self.enabled {
                            // Insert new sensor value to cache
                            let update2 = update.clone();
                            cache.insert(update2.sensor.name, update2.value);
                            debug!(
                                "{} {}: {:?} cache updated.",
                                &update.device_name, &update.sensor.name, &update.value
                            );
                        }
                    }
                }
                None => todo!(),
            };
        }
    }
}
