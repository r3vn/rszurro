use crate::{update_sensor_sync, Endpoint, Sensor, SensorValue};
use lm_sensors::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, thread, time};

#[derive(Deserialize, Serialize, Debug)]
pub struct LMSensors {
    pub enabled: bool,
    pub sleep_ms: u64,
    pub device_name: String,
    pub temperature_unit: String,
}
impl LMSensors {
    pub fn run(
        &self,
        endpoints: Vec<Endpoint>,
        verbosity: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize last value map
        let mut last_value_map: HashMap<String, f64> = HashMap::new();

        // Initialize LM sensors library.
        let sensors = lm_sensors::Initializer::default().initialize().unwrap();

        loop {
            // Get all chips from lm-sensors.
            for chip in sensors.chip_iter(None) {
                // Get Chip name
                let chip_name = chip.prefix().unwrap().unwrap();

                // Get all features of the current chip.
                for feature in chip.feature_iter() {
                    // Set device class from feature kind
                    let device_class = match feature.kind() {
                        Some(lm_sensors::feature::Kind::Temperature) => "temperature".to_string(),
                        Some(lm_sensors::feature::Kind::Humidity) => "humidity".to_string(),
                        Some(lm_sensors::feature::Kind::Voltage) => "voltage".to_string(),
                        Some(lm_sensors::feature::Kind::Power) => "power".to_string(),
                        Some(lm_sensors::feature::Kind::Current) => "current".to_string(),
                        _ => "None".to_string(),
                    };

                    // Set unit from device_class
                    let unit = match device_class.as_str() {
                        "temperature" => self.temperature_unit.clone(),
                        "humidity" => "%".to_string(),
                        "voltage" => "V".to_string(),
                        "power" => "W".to_string(),
                        _ => "".to_string(),
                    };

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

                            // trunc float value to one digit
                            let float_value = f64::trunc(value.raw_value() * 10.0) / 10.0;

                            // check if value changed since last iteration
                            if last_value_map.get(&sensor_name_str) != Some(&float_value) {
                                // Value changed since last iteration
                                if verbosity > 1 {
                                    println!(
                                        "[lm_sensors] chip: {} sensor: {} - value changed sending to endpoints...", 
                                        chip_name, sub_feature);
                                }

                                // Build a sensor object
                                let sensor = Sensor {
                                    name: format!("{}_{}", &chip_name, sensor_name_str),
                                    friendly_name: format!(
                                        "{}'s {}",
                                        &self.device_name, sensor_name_str
                                    ),
                                    unit: unit.clone(),
                                    accuracy: 1.0,
                                    address: 0,
                                    is_bool: false,
                                    state_class: "measurement".to_string(),
                                    device_class: device_class.clone(),
                                };

                                // Send value to Home Assistant
                                update_sensor_sync(
                                    &endpoints,
                                    &self.device_name,
                                    &sensor,
                                    SensorValue::IsF64(float_value),
                                );
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
