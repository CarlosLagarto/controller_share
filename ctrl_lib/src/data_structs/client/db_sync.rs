use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::app_time::ctrl_time::*;
use crate::data_structs::client::{client_ctx::*, cycle_cli::*, sector_cli::*, sync_type::*};
use crate::data_structs::msgs::{ext_message::*, topic::*};
use ctrl_prelude::{domain_types::*, globals::*};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SensorMsg {
    // SPRINT SENSORES  - TBD no sprint dos sensores
}

// The client have the cycle and and physical sectors lists
//
// The cycle list shows the active cycles, except the direct ones, which might appear in the history
// The sectors listshows the physical sectors and their state, watering or not
pub type SecCliList = ArrayVec<SectorCli, MAX_SECTORS>;
pub type CycleCliList = ArrayVec<CycleCli, MAX_CYCLES>;

/// Dimension = 1376
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DBSync {
    pub last_sync: UTC_UNIX_TIME,
    pub sync_type: String,
    pub sensors: Option<Vec<SensorMsg>>,
    pub cycles: CycleCliList,
    pub cycle_history_last_week: Option<CycleCliList>,
    pub sectors: SecCliList,
    pub config: Option<ClientCtx>,
}

impl DBSync {
    #[inline]
    #[rustfmt::skip]
    pub fn new_out(db_sync: DBSync, time: CtrlTime) -> ExtMsgOut {
        ExtMsgOut::new(
            Topic::STC_SYNC_DB,
            ExtMsgOut::DBSync(Box::new(DBSyncMsg { header: None, db_sync: Box::new(db_sync) })),
            time,
        )
    }

    #[inline]
    pub fn build_full(time: CtrlTime, cfg: &ClientCtx, cycles: CycleCliList, sectors: SecCliList) -> DBSync {
        DBSync {
            sync_type: SyncType::FULL.to_str(),
            last_sync: time.ux_ts(),
            cycles,
            sectors,
            sensors: None,
            config: Some(ClientCtx::from_context(cfg)),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{data_structs::msgs::ext_message::ExtMsgIn, lib_serde::data_from_str};

    #[test]
    fn test_db_sync() {
        let msg = r#"{
                                "header": {
                                    "uuid": "f48e10c0-5945-11ed-82e8-23a98829e780",
                                    "client_id": "laptop-ax-01",
                                    "time": 1667239250,
                                    "topic": "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST"
                                },
                                "type": "DBSync",
                                "db_sync": {
                                    "sync_type": "P",
                                    "last_sync": 1667239250,
                                    "cycles": [
                                        {
                                            "name": "Wizard-auto",
                                            "cycle_id": 111158,
                                            "start": 16672791540,
                                            "run_start": 16672791540,
                                            "end": 16672791640,
                                            "run_id": 1,
                                            "last_run": 0,
                                            "status": "Waiting",
                                            "repeat_kind": "Every",
                                            "repeat_spec_wd": 0,
                                            "repeat_every_qty": 1,
                                            "repeat_every_unit": "Days",
                                            "stop_condition": "Never",
                                            "stop_retries": 0,
                                            "stop_date_ts": 0,
                                            "retries_count": 0,
                                            "sunrise_flg": 1,
                                            "sunset_flg": 0,
                                            "cycle_type": "Wizard",
                                            "last_change": 16669979640,
                                            "op": "I"
                                        },
                                        {
                                            "name": "direct",
                                            "cycle_id": 111159,
                                            "start": 16669979640,
                                            "run_start": 16672791540,
                                            "end": 16672791640,
                                            "run_id": 2,
                                            "last_run": 0,
                                            "status": "Waiting",
                                            "repeat_kind": "Never",
                                            "repeat_spec_wd": 0,
                                            "repeat_every_qty": 0,
                                            "repeat_every_unit": "Seconds",
                                            "stop_condition": "Retries",
                                            "stop_retries": 1,
                                            "stop_date_ts": 0,
                                            "retries_count": 0,
                                            "sunrise_flg": 0,
                                            "sunset_flg": 0,
                                            "cycle_type": "Wizard",
                                            "last_change": 16669979640,
                                            "op": "I"
                                        }
                                    ],
                                    "sectors": [
                                        {
                                            "id": 0,
                                            "desc": "Zona Sobreiro",
                                            "deficit": -273425.88,
                                            "percolation": 0.001666667,
                                            "debit": 0.4333333,
                                            "last_watered_in": 16429172410,
                                            "start": 16429172410,
                                            "end": 16429182410,
                                            "enabled": true,
                                            "max_duration": 30,
                                            "minutes_to_water": 0,
                                            "status": "Waiting",
                                            "name": "Sobreiro",
                                            "last_change": 16429172410,
                                            "op": "I",
                                            "stress_score": 0,
                                            "stress_perc": 7656024
                                        }
                                    ]
                                }
                            }"#;

        let deserialized: ExtMsgIn = data_from_str(&msg).unwrap();
        println!("{:?}", deserialized);
    }
}
