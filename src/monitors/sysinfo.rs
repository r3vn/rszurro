use crate::{update_sensor, Sensor, Endpoint};
use serde_derive::{Deserialize, Serialize};
use tokio::{time::sleep, fs, time::Duration};

#[derive(Deserialize, Serialize, Debug)]
pub struct SysInfo {
    pub enabled: bool,
    pub device_name: String,
    pub uptime: bool,
    pub loadavg: bool,
    pub sleep_ms: u64
}

impl SysInfo {
    pub async fn run(
        &self,
        endpoints: Vec<Endpoint>,
        verbosity: u8
    ) -> Result<(), Box<dyn std::error::Error>> {

        loop {
            // uptime
            if self.uptime {
                let uptime = fs::read_to_string("/proc/uptime")
                    .await?;

                let uptime_seconds: f64 = uptime
                    .split('.')
                    .next()
                    .and_then(|u| u.parse().ok())
                    .ok_or("[sysinfo] error: unable to read uptime.")?;

                let uptime_sensor = Sensor {
                    name: "uptime".to_string(),
                    friendly_name: format!("{}'s uptime", &self.device_name),
                    unit: "s".to_string(),
                    accuracy: 1.0,
                    address: 0,
                    state_class: "".to_string(),
                    device_class: "duration".to_string()
                };

                if verbosity > 1 {
                    println!("[sysinfo] uptime: {}, sending to endpoints...", &uptime_seconds)
                }

                update_sensor(
                    &endpoints,
                    &self.device_name,
                    &uptime_sensor,
                    uptime_seconds
                ).await;
            }

            // sleep
            sleep(Duration::from_millis(self.sleep_ms)).await; 
        }
    }
}
