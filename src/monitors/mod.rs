// lm_sensors
#[cfg(feature = "lmsensors")]
pub use self::lm_sensors_monitor::LMSensors;

#[cfg(feature = "lmsensors")]
mod lm_sensors;

#[cfg(feature = "lmsensors")]
pub mod lm_sensors_monitor {
    pub use crate::monitors::lm_sensors::LMSensors;
}

// modbus_rtu
#[cfg(feature = "modbus-rtu")]
pub use self::modbus_rtu_monitor::ModbusRTU;

#[cfg(feature = "modbus-rtu")]
mod modbus_rtu;

#[cfg(feature = "modbus-rtu")]
pub mod modbus_rtu_monitor {
    pub use crate::monitors::modbus_rtu::ModbusRTU;
}

// sysinfo
#[cfg(feature = "sysinfo")]
pub use self::sysinfo_monitor::SysInfo;

#[cfg(feature = "sysinfo")]
mod sysinfo;

#[cfg(feature = "sysinfo")]
pub mod sysinfo_monitor {
    pub use crate::monitors::sysinfo::SysInfo;
}

// gpio
#[cfg(feature = "gpio")]
pub use self::gpio_monitor::Gpio;

#[cfg(feature = "gpio")]
mod gpio;

#[cfg(feature = "gpio")]
pub mod gpio_monitor {
    pub use crate::monitors::gpio::Gpio;
}
