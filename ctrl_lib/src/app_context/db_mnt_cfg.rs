use crate::app_time::ctrl_time::*;

/// Dimension = 10 aligned to 16
pub struct DBMntCfg {
    pub db_mnt_last_run: CtrlTime, 
    pub db_mnt_counter: u8,        
    pub db_mnt_days: u8,           
}
