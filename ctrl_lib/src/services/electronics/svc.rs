use num_enum::TryFromPrimitive;
use strum_macros::Display;

use crate::{log_debug, log_error, logger::*, ifmt, app_time::ctrl_time::sim_bool};
use ctrl_prelude::{domain_types::*, string_resources::*};
use crate::utils::conv_int;

// SPRINT IO
#[allow(non_camel_case_types)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Display, Copy, Clone, PartialEq, TryFromPrimitive)]
#[repr(i8)]
pub enum VALVE_STATE {
    OPEN = 1,   //its watering
    CLOSED = 0, //its not watering
    ERROR = -1,
}

impl Default for VALVE_STATE {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self { VALVE_STATE::CLOSED }
}

#[rustfmt::skip]
impl VALVE_STATE {
    #[inline]
    pub fn is_closed(&self) -> bool { *self == VALVE_STATE::CLOSED }

    #[inline]
    pub fn is_error(&self) -> bool { *self == VALVE_STATE::ERROR }

    #[inline]
    pub fn is_open(&self) -> bool { *self == VALVE_STATE::OPEN }

    #[inline]
    pub fn is_error_or_open(&self)->bool{ *self == VALVE_STATE::ERROR || *self == VALVE_STATE::OPEN }

    #[inline]
    pub fn is_error_or_closed(&self)->bool{ *self == VALVE_STATE::ERROR || *self == VALVE_STATE::CLOSED }

}

//SPRINT IO - rever otimização mas só depois de isto estar feito :-)
#[allow(non_snake_case)]
#[inline]
fn IO(sector_id: SECTOR_ID, on: bool) -> bool {
    // default on = false}
    // input/Output
    let mut result = false;
    if sim_bool() {
        result = true; //# não faz nada, podemos pensar em fazer uma simulação qualquer
                       // REVER
        if on {
            log_debug!(dbg_wtr_adptr_simul_valve_open(&ifmt!(sector_id)));
        } else {
            log_debug!(dbg_wtr_adptr_simul_valve_closed(&ifmt!(sector_id)));
        }
    } else {
        //# é o que vai chamar os relés
        if on {
            log_debug!(dbg_wtr_adptr_valve_open(&ifmt!(sector_id)));
        } else {
            log_debug!(dbg_wtr_adptr_valve_closed(&ifmt!(sector_id)));
        }
    }
    result
}

#[inline]
#[rustfmt::skip]
fn log_critical_erro(is_err: bool, _err: String) {
    if is_err { log_error!(PHADPT_CRITICAL_IO); } //REVER- não sei ainda o que fazer.... 
}

#[inline]
#[rustfmt::skip]
pub fn turn_on_physical_sec(sector_id: SECTOR_ID) -> bool {
    //# interface com a parte fisica para ligar a válvula do setor
    let result = IO(sector_id,  true);
    if !result { log_critical_erro(result, String::new()); }
    result
}

#[inline]
#[rustfmt::skip]
pub fn turn_off_physical_sec(sector_id: SECTOR_ID) -> bool {
    let result = IO(sector_id,  false);
    if !result { log_critical_erro(result, String::new()); }
    result
}

#[inline]
pub fn valve_status(_sector_id: SECTOR_ID) -> VALVE_STATE {
    //há-de haver uma forma de interrogar o sistema para saber o estado
    VALVE_STATE::CLOSED
    //REVIEW para testar isto sem maquina ligada ainda vou ter que pensar em algo
}
