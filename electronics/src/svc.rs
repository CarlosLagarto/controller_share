use num_enum::TryFromPrimitive;
use strum_macros::Display;

use ctrl_lib::{log_debug, log_error, logger::*, app_time::ctrl_time::sim_bool};
use ctrl_prelude::{domain_types::*, string_resources::*};

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
}

#[allow(non_snake_case)]
#[inline]
fn IO(sector_id: SECTOR_ID, on: bool) -> bool {
    //default on = false}
    // input/Output
    let mut result = false;
    if sim_bool() {
        //# não faz nada, podemos pensar em fazer uma simulação qualquer
        result = true;
        // TODO: IO SPRINT
        let msg = if on { dbg_wtr_adptr_simul_valve_open(&sector_id.to_string()) } else { dbg_wtr_adptr_simul_valve_closed(&sector_id.to_string()) };
        #[cfg(debug_assertions)]
        log_debug!(msg);
    } else {
        // TODO: IO SPRINT
        //# é o que vai chamar os relés
        let msg = if on { dbg_wtr_adptr_simul_valve_open(&sector_id.to_string()) } else { dbg_wtr_adptr_simul_valve_closed(&sector_id.to_string()) };
        #[cfg(debug_assertions)]
        log_debug!(msg);
    }
    result
}

#[inline]
fn log_critical_error(is_err: bool, _err: &str) {
    if is_err {
        log_error!(PHADPT_CRITICAL_IO); // TODO: IO SPRINT - não sei ainda o que fazer....
    }
}

#[inline]
pub fn turn_on_physical_sector(sector_id: SECTOR_ID) -> bool {
    //# interface com a parte fisica para ligar a válvula do setor
    let result = IO(sector_id,  true);
    log_critical_error(result, "");
    result
}

#[inline]
pub fn turn_off_physical_sector(sector_id: SECTOR_ID) -> bool {
    let result = IO(sector_id,false);
    log_critical_error(result, "");
    result
}

#[inline]
pub fn valve_status(_sector_id: SECTOR_ID) -> VALVE_STATE {
    //há-de haver uma forma de interrogar o sistema para saber o estado
    VALVE_STATE::CLOSED

    // TODO IO SPRINT - para testar isto sem maquina ligada ainda vou ter que pensar em algo
}
