use crate::utils::deserialize_file;

const APP_LOG_CFG_FILE: &str = "data/log_config.toml";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppLog {
    pub(crate) directory: String,
    pub(crate) file: String,
    pub(crate) level: String,
    pub(crate) nr_of_days_to_maintain_log_files: u8,
    pub(crate) mb_limit: u32,
    // regarding the global design, this may not be in the best place, but it was a quick solution to segregate tst and prd environments
    // so i can keep developping in the same machine where the production progeam is running
    // Everything is segregated based on the program path
    // The things that need this configuration flag are:
    //  - single instance validation, because it uses a port to check, and so despite the different path, we needed a flag
    //  - mqtt, to know if we subscribe/publish to the topics with the TEST suffix or not
    //  - weather, as we only have one station, so only one UDP port, so we have to listen to different ports or use a different mechanism
    //  - web server, that have to listen in different ports, despite the path
    pub test_in_progress : u8,
}

impl AppLog {
    #[inline]
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    pub fn new() -> Self { deserialize_file::<AppLog>(APP_LOG_CFG_FILE) }
}
