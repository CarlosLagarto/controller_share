#![allow(non_camel_case_types)]

/// High Resolution está no mesmo u64, porque pelos requisitos que defini, chega para ter o nr de nano secs entre 1970 e 2077
pub type APP_TIME = u64;
pub type UTC_UNIX_TIME_HR = f64;
pub type UTC_ISO_DATE_STR = String;
/// como defininos o requisito que a aplicação só trabalha com tempos depois de 1970, u64 mostra essa intenção
pub type UTC_UNIX_TIME = u64;
pub type STD_DURATION = std::time::Duration;

pub type CYCLE_RUN = u32;
pub type CYCLE_ID = u32;
pub type DEVICE_ID = u16;  // se tiver mais do que 65000 devices, era uma festa.  Espero muitos, 
pub type SCENE_ID = u16;
pub type CYCLE_PTR = u8;
pub type SECTOR_ID = u8;
pub type SECTOR_PTR = u8;
pub type SUN_FLAG = u8; 
pub type REPEAT_QTY = u16;
pub type RETRIES = u16;
pub type SENSOR_ID_T = u8;
// pub type SIM = u8;
pub type WARP = u8;
pub type DUR = i64;

