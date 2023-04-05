use crate::app_time::ctrl_time::*;

/// Dimension = 40
#[derive(Clone)]
pub struct StartupData {
    pub start_date_str: String,
    pub start_date: CtrlTime,

}

impl Default for StartupData {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self {
        let start_date = CtrlTime::sys_time();
        Self { start_date, start_date_str: String::new() }
    }
}

#[rustfmt::skip]
impl StartupData {

    #[inline]
    pub fn build(start_date: CtrlTime) -> Self {
        Self { start_date, start_date_str: start_date.as_rfc3339_str_e() }
    }

    // by use case definition we only work in the definied time interval.
    // CtrlTime was tested from 1970 (CtrlTime::MIN_YEAR) to 2077 (CtrlTime::Max_YEAR), but the program works only within the defined interval of 2022/jan to 2070/dec
    #[inline]
    pub fn is_valid(&self) -> bool { 
        let valid_inf_date = self.start_date >= CtrlTime::from_utc_parts(2022, 1,  1 , 0, 0, 0);
        let valid_sup_date = self.start_date <= CtrlTime::from_utc_parts(2070, 12,  31 , 23, 59, 59);
        valid_inf_date && valid_sup_date
    }
}
