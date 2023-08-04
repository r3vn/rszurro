// lm_sensors
pub use self::lm_sensors_monitor::LMSensors;
mod lm_sensors;
pub mod lm_sensors_monitor {
    pub use crate::monitors::lm_sensors::LMSensors;
}

// modbus_rtu
pub use self::modbus_rtu_monitor::ModbusRTU;
mod modbus_rtu;
pub mod modbus_rtu_monitor {
    pub use crate::monitors::modbus_rtu::ModbusRTU;
}
