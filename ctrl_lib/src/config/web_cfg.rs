use serde::{self, Deserialize, Serialize};

use crate::utils::deserialize_file;

const WEB_CFG_FILE: &str = "data/web_rest_config.toml";

/// Dimension = 56
#[derive(Clone, Serialize, Deserialize)]
pub struct WebCfg {
    pub port: u16,
    pub address: String,
    pub test_port: u16,
    pub test_address: String,
}

impl WebCfg {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        deserialize_file::<WebCfg>(WEB_CFG_FILE)
    }
}
