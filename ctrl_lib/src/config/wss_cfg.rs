use serde::{self, Deserialize, Serialize};

use crate::utils::deserialize_file;

const WSS_CFG_FILE: &str = "data/wss_config.toml";

/// Dimension = 32
#[derive(Clone, Serialize, Deserialize)]
pub struct WSSCfg {
    pub port: u16,
    pub address: String, 
}

impl WSSCfg {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { deserialize_file::<WSSCfg>(WSS_CFG_FILE) }
}
