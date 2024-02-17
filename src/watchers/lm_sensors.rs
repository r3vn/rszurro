use log::trace;
use std::{thread, time};
use tokio::sync::mpsc;

use crate::{update_sensor_sync, Sensor, SensorUpdate, SensorValue, Watcher};

pub fn run(
    watcher: Watcher,
    tx: mpsc::Sender<SensorUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
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
                    "temperature" => watcher.temperature_unit.clone(),
                    "humidity" => "%".to_string(),
                    "voltage" => "V".to_string(),
                    "power" => "W".to_string(),
                    _ => "".to_string(),
                };

                // Get all sub-features of the current chip feature.
                for sub_feature in feature.sub_feature_iter() {
                    if let Ok(value) = sub_feature.value() {
                        // Sensor has value
                        trace!("{} {} => {}", chip_name, sub_feature, value);

                        // get sensor name from lmsensors
                        let sensor_name_str = sub_feature.name().unwrap().unwrap().to_string();

                        // trunc float value to one digit
                        let float_value = f64::trunc(value.raw_value() * 10.0) / 10.0;

                        // Build a sensor object
                        let sensor = Sensor {
                            name: format!("{}_{}", &chip_name, sensor_name_str),
                            friendly_name: format!("{}'s {}", &watcher.name, sensor_name_str),
                            unit: unit.clone(),
                            accuracy: 1.0,
                            address: 0,
                            debounce_delay: 0,
                            state_class: "measurement".to_string(),
                            device_class: device_class.clone(),
                        };

                        // Send value to Home Assistant
                        update_sensor_sync(
                            &tx,
                            &watcher.name,
                            &sensor,
                            SensorValue::IsF64(float_value),
                        );
                    }
                }
            }
            // sleep between readings
            thread::sleep(time::Duration::from_millis(watcher.scan_interval));
        }
    }
}
