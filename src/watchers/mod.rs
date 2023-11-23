// lm_sensors
#[cfg(feature = "lmsensors")]
pub use self::lm_sensors_watcher::LMSensors;

#[cfg(feature = "lmsensors")]
mod lm_sensors;

#[cfg(feature = "lmsensors")]
pub mod lm_sensors_watcher {
    pub use crate::watchers::lm_sensors::LMSensors;
}

// modbus_rtu
#[cfg(feature = "modbus-rtu")]
pub use self::modbus_rtu_watcher::ModbusRTU;

#[cfg(feature = "modbus-rtu")]
mod modbus_rtu;

#[cfg(feature = "modbus-rtu")]
pub mod modbus_rtu_watcher {
    pub use crate::watchers::modbus_rtu::ModbusRTU;
}

// sysinfo
#[cfg(feature = "sysinfo")]
pub use self::sysinfo_watcher::SysInfo;

#[cfg(feature = "sysinfo")]
mod sysinfo;

#[cfg(feature = "sysinfo")]
pub mod sysinfo_watcher {
    pub use crate::watchers::sysinfo::SysInfo;
}

// gpio
#[cfg(feature = "gpio")]
pub use self::gpio_watcher::Gpio;

#[cfg(feature = "gpio")]
mod gpio;

#[cfg(feature = "gpio")]
pub mod gpio_watcher {
    pub use crate::watchers::gpio::Gpio;
}
