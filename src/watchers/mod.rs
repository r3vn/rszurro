#[cfg(feature = "lmsensors")]
pub mod lm_sensors;

#[cfg(feature = "modbus-rtu")]
pub mod modbus_rtu;

#[cfg(feature = "sysinfo")]
pub mod sysinfo;

#[cfg(feature = "gpio")]
pub mod gpio;
