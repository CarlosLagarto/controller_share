use serde::{self, Deserialize, Serialize};

use crate::utils::deserialize_file;

const MQTT_CONFIG_FILE: &str = "data/mqtt_config.toml";

/// Dimension = 80 
#[derive(Serialize, Deserialize)]
pub struct MQTTConfig {
    pub broker_port: u16,
    pub web_broker_port: u16,
    pub broker_address: String,     
    pub client_id: String,         
    pub web_broker_address: String, 
    pub wait_mqtt_chk_interval: u8,
}

impl MQTTConfig {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        deserialize_file::<MQTTConfig>(MQTT_CONFIG_FILE)
    }
}


