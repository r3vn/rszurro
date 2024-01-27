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
            .ping(address, 6969, 1, Duration::from_millis(1000))
            .await
        {
            Ok(Some(_)) => true,
            Ok(..) => false,
            Err(_) => false,
        };

        // make a sensor
        let sensor = Sensor {
            name: "status".to_string(),
            friendly_name: watcher.name.clone(),
            address: 0,
            is_bool: false,
            accuracy: 0.0,
            unit: "".to_string(),
            state_class: "".to_string(),
            device_class: "".to_string(),
        };

        // update sensor cache
        update_sensor(&tx, &watcher.name, &sensor, SensorValue::IsBool(value)).await;

        // sleep for next update
        sleep(Duration::from_millis(watcher.scan_interval)).await;
    }
}
