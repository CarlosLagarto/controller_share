use serde::{Deserialize, Serialize};

use crate::data_structs::rega::watering_status::*;
use crate::db::{db_sql_lite::*, *};
use crate::services::irrigation::{cycle_type::*, db_model::*};
use crate::{lib_serde::*, log_error, logger::*};
use ctrl_prelude::{domain_types::*, globals::*};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorHstry {
    pub id_sector: SECTOR_ID,
    pub minutes_to_water_acc: f32,
    pub status: WateringStatus,
    pub name: String,
    pub start: UTC_UNIX_TIME,
    pub end: UTC_UNIX_TIME,
}

impl From<&SqlRow<'_>> for SectorHstry {
    #[inline]
    fn from(sql_row: &SqlRow) -> SectorHstry {
        let sql_row = sql_row;
        SectorHstry {
            id_sector: sql_row.get(0).unwrap(),
            minutes_to_water_acc: sql_row.get(1).unwrap(),
            status: unsafe { WateringStatus::from_unchecked(sql_row.get(2).unwrap()) },
            name: sql_row.get(3).unwrap(),
            start: sql_row.get(4).unwrap(),
            end: sql_row.get(5).unwrap(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleHstry {
    pub cycleid: CYCLE_ID,
    pub cycle_type: CycleType,
    pub current_run: u32,
    pub name: String,
    pub status: WateringStatus,
    start: UTC_UNIX_TIME,
    end: UTC_UNIX_TIME,
    pub sectors: Vec<SectorHstry>,
}

impl From<&SqlRow<'_>> for CycleHstry {
    #[inline]
    fn from(sql_row: &SqlRow) -> CycleHstry {
        let sql_row = sql_row;
        CycleHstry {
            cycleid: sql_row.get(0).unwrap(),
            cycle_type: unsafe { CycleType::from_unchecked(sql_row.get(1).unwrap()) },
            current_run: sql_row.get(2).unwrap(),
            name: sql_row.get(3).unwrap(),
            status: unsafe { WateringStatus::from_unchecked(sql_row.get(4).unwrap()) },
            start: sql_row.get(5).unwrap(),
            end: sql_row.get(6).unwrap(),
            sectors: Vec::with_capacity(MAX_SECTORS),
        }
    }
}

/// Dimension = 152
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterHstry {
    pub cycles: Vec<CycleHstry>,
}

impl WaterHstry {
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    #[inline]
    pub fn new() -> WaterHstry { WaterHstry { cycles: Vec::new() } }

    #[inline]
    pub fn build(time: UTC_UNIX_TIME, db: &Persist) -> Option<WaterHstry> {
        let result = db.get_water_history(time);
        match result {
            Ok(data) => Some(data),
            Err(e) => {
                log_error!(e);
                None
            }
        }
    }
}

impl Json for WaterHstry {
    #[inline]
    #[rustfmt::skip]
    fn json(&self) -> JsonResult<String> { data_to_str(&self) }
}
