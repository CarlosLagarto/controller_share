use super::db_model::DBModelRega;
use crate::db::db_sql_lite::*;

#[inline]
pub fn system_check_and_recover(db: &Persist) {
    let db = db;
    // valid state at start up is running, or terminated, or error.
    // Inconsistent data have the problem of not knowing when the thing went wrong.
    // A good proxy is the start time
    // There may exist a deficit in the water level.
    // We will live with that "error" as after, at maximum one week, the wizard will balance
    // It seems that any other solution will have an arbitrary starting point.
    let _res = db.recover_inconsistent_cycles();
    let _res = db.recover_cycle_run();
    let _res = db.recover_secs_run();

    // SPRINT: IO o mesmo para os IO,s - ver se á algum eventType de teste que se possa fazer no arranque da máquina
    // Na meteoreologia em tese não é necessário
    // Já para os sensores, SPRINT: - SENSORS  qd os houver, pode-se também fazer uma verification check, onde possivel, no startup
    // Mas os sensores, os SPRINT - IO IOs e a metereologia, devem ter este tema endereçado nos seus métodos de start.
}
