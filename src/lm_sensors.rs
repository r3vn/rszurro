use crate::{Homeassistant, Sensor};
use lm_sensors::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, thread, time};

#[derive(Deserialize, Serialize, Debug)]
pub struct Monitor {
    pub enabled: bool,
    pub sleep_ms: u64,
    pub device_name: String,
    pub unit: String,
}
impl Monitor {
    pub fn run(
        &self,
        homeassistant: &Homeassistant,
        verbosity: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize LM sensors library.
        let sensors = lm_sensors::Initializer::default().initialize().unwrap();

        // Initialize last value map
        let mut last_value_map: HashMap<String, f64> = HashMap::new();

        loop {
            // Get all chips from lm-sensors.
            for chip in sensors.chip_iter(None) {
                // Get Chip name
                let chip_name = chip.prefix().unwrap().unwrap();

                // Get all features of the current chip.
                for feature in chip.feature_iter() {
                    // Get all sub-features of the current chip feature.
                    for sub_feature in feature.sub_feature_iter() {
                        if let Ok(value) = sub_feature.value() {
                            // Sensor has value
                            if verbosity > 2 {
                                println!(
                                    "[lm_sensors] chip: {} sensor: {}: {}",
                                    chip_name, sub_feature, value
                                );
                            }

                            // get sensor name from lmsensors
                            let sensor_name_str =
                                sub_feature.clone().name().unwrap().unwrap().to_string();

                            // trunc float value to two digits
                            let float_value = f64::trunc(value.raw_value() * 100.0) / 100.0;

                            // check if value changed since last iteration
                            if last_value_map.get(&sensor_name_str) != Some(&float_value) {
                                // Value changed since last iteration
                                if verbosity > 1 {
                                    println!(
                                        "[lm_sensors] chip: {} sensor: {} - value changed sending to HA...", 
                                        chip_name, sub_feature);
                                }

                                // Build a sensor object
                                let sensor = Sensor {
                                    name: format!("{}_{}", &chip_name, sensor_name_str),
                                    friendly_name: format!(
                                        "{}'s {}",
                                        &self.device_name, sensor_name_str
                                    ),
                                    unit: self.unit.clone(),
                                    accuracy: 1.0,
                                    address: 0,
                                    state_class: "measurement".to_string(),
                                    device_class: "temperature".to_string(),
                                };

                                // Send value to Home Assistant
                                let ha_rx = homeassistant.send_sync(
                                    &self.device_name,
                                    &sensor,
                                    float_value,
                                );

                                // check home assistant response
                                if ha_rx && verbosity > 1 {
                                    // Sensor's value sent to home assistant successfully
                                    println!(
                                        "[lm_sensors] chip: {} sensor: {} - done.",
                                        &chip_name, &sensor_name_str
                                    );
                                } else if !ha_rx && verbosity > 0 {
                                    // Error sending sensor's value to home assistant
                                    println!(
                                        "[lm_sensors] chip: {} sensor: {} - error, sleeping...",
                                        &chip_name, &sensor_name_str
                                    );

                                    // Sleep for a while...
                                    thread::sleep(time::Duration::from_millis(self.sleep_ms * 2));
                                }
                            }

                            // update last value map
                            last_value_map.insert(sensor_name_str, float_value);
                        }
                    }
                }
                // sleep between readings
                thread::sleep(time::Duration::from_millis(self.sleep_ms));
            }
        }
    }
}
