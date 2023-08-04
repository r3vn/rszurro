// homeassistant
pub use self::homeassistant_endpoint::Homeassistant;
mod homeassistant;
pub mod homeassistant_endpoint {
    pub use crate::endpoints::homeassistant::Homeassistant;
}
