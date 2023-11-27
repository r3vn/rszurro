use log::trace;
use tokio::sync::mpsc;
use tokio::{fs, time::sleep, time::Duration};

use crate::{update_sensor, Sensor, SensorUpdate, SensorValue, Watcher};

pub async fn run(
    watcher: Watcher,
    tx: mpsc::Sender<SensorUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // uptime
        let uptime = fs::read_to_string("/proc/uptime").await?;

        let uptime_seconds: f64 = uptime
            .split('.')
            .next()
            .and_then(|u| u.parse().ok())
            .ok_or("error")?;

        let uptime_sensor = Sensor {
            name: "uptime".to_string(),
            friendly_name: format!("{}'s uptime", &watcher.name),
            unit: "s".to_string(),
            accuracy: 1.0,
            address: 0,
            is_bool: false,
            state_class: "".to_string(),
            device_class: "duration".to_string(),
        };

        trace!("uptime => {}", &uptime_seconds);

        update_sensor(
            &tx,
            &watcher.name,
            &uptime_sensor,
            SensorValue::IsF64(uptime_seconds),
        )
        .await;

        // sleep
        sleep(Duration::from_millis(watcher.sleep_ms)).await;
    }
}
