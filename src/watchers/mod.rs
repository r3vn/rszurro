// lm_sensors
#[cfg(feature = "lmsensors")]
pub mod lm_sensors;

#[cfg(feature = "modbus-rtu")]
pub mod modbus_rtu;

// sysinfo
#[cfg(feature = "sysinfo")]
pub mod sysinfo;

// gpio
#[cfg(feature = "gpio")]
pub mod gpio;
