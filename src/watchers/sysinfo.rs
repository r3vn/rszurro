use crate::{update_sensor, Sensor, SensorValue, SensorUpdate};
use serde_derive::{Deserialize, Serialize};
use tokio::{fs, time::sleep, time::Duration};
use tokio::sync::mpsc;
use log::trace;

#[derive(Deserialize, Serialize, Debug)]
pub struct SysInfo {
    pub enabled: bool,
    pub device_name: String,
    pub uptime: bool,
    pub loadavg: bool,
    pub sleep_ms: u64,
}

impl SysInfo {
    pub async fn run(
        &self,
        tx: mpsc::Sender<SensorUpdate>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // uptime
            if self.uptime {
                let uptime = fs::read_to_string("/proc/uptime").await?;

                let uptime_seconds: f64 = uptime
                    .split('.')
                    .next()
                    .and_then(|u| u.parse().ok())
                    .ok_or("error")?;

                let uptime_sensor = Sensor {
                    name: "uptime".to_string(),
                    friendly_name: format!("{}'s uptime", &self.device_name),
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
                    &self.device_name,
                    &uptime_sensor,
                    SensorValue::IsF64(uptime_seconds),
                )
                .await;
            }

            // load avg
            if self.loadavg {
                let _load = fs::read_to_string("/proc/loadavg").await?;
            }

            // sleep
            sleep(Duration::from_millis(self.sleep_ms)).await;
        }
    }
}
