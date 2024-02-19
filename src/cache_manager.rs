use log::{debug, error, info, trace};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use crate::{Endpoint, EndpointConnection, SensorUpdate};

pub struct CacheManager {
    pub enabled: bool,
    pub endpoints: Vec<Endpoint>,
}
impl CacheManager {
    pub async fn run(&self, mut rx: mpsc::Receiver<SensorUpdate>) {
        // init cache
        let mut cache = HashMap::new();

        // get endpoints clients if any
        let mut connections = HashMap::new();

        for endpoint in &self.endpoints {
            let state = Arc::new(Mutex::new(true));

            connections
                .entry(&endpoint.name)
                .or_insert(EndpointConnection {
                    client: endpoint.get_client(state.clone()).await,
                    state,
                });
        }

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
                    let last_value =
                        cache.get(&self.get_key(&update.device_name, &update.sensor.name).await);

                    // check if value changed from the cached one
                    if last_value != Some(&update.value) {
                        for endpoint in &self.endpoints {
                            // clone update and endpoint's client
                            let update1 = update.clone();
                            let connection = connections.get(&endpoint.name).unwrap();

                            if !(*connection.state.lock().await) {
                                error!("Retrying connection to {}...", &endpoint.name);

                                let state = Arc::new(Mutex::new(true));
                                connections.insert(
                                    &endpoint.name,
                                    EndpointConnection {
                                        client: endpoint.get_client(state.clone()).await,
                                        state,
                                    },
                                );
                            }

                            let client = connections.get(&endpoint.name).unwrap().client.clone();

                            // send data to endpoint
                            endpoint.run(update1, client).await;
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
                            cache.insert(
                                self.get_key(&update2.device_name, &update2.sensor.name)
                                    .await,
                                update2.value,
                            );
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

    async fn get_key(&self, device_name: &String, sensor_name: &String) -> String {
        format!("{}/{}", device_name, sensor_name)
    }
}
