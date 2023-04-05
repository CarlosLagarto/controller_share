use serde::{Deserialize, Serialize};

use crate::utils::deserialize_file;

const DB_CONFIG_FILE: &str = "data/db_config.toml";

/// Dimension 24 + 40 no heap
#[derive(Serialize, Deserialize)]
pub struct DBConfig {
    pub db_name: String,
    pub dw_name: String,
}

impl DBConfig {
    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { deserialize_file::<DBConfig>(DB_CONFIG_FILE) }
}
