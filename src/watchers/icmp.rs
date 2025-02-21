use std::net::IpAddr;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_icmp_echo::Pinger;

use crate::{update_sensor, Sensor, SensorUpdate, SensorValue, Watcher};

pub async fn run(
    watcher: Watcher,
    tx: mpsc::Sender<SensorUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = IpAddr::from_str(&watcher.host).unwrap();
    let pinger = Pinger::new().await.unwrap();

    loop {
        // ping host
        let value = match pinger
            .ping(
                address,
                6969,
                watcher.count.try_into().unwrap(),
                Duration::from_millis(watcher.timeout),
            )
            .await
        {
            Ok(Some(_)) => true,
            Ok(..) => false,
            Err(_) => false,
        };

        // make a sensor
        let sensor = Sensor::new("status".to_string(), watcher.name.clone()).await;

        // update sensor cache
        update_sensor(&tx, &watcher.name, &sensor, SensorValue::IsBool(value)).await;

        // sleep for next update
        sleep(Duration::from_millis(watcher.scan_interval)).await;
    }
}
