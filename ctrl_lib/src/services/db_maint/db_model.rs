use std::time::Instant;

use crate::app_time::ctrl_time::*;
use crate::db::{db_error::*, db_sql_lite::*};
use crate::utils::elapsed_dyn;
use crate::{log_error, logger::*};
use ctrl_prelude::error::build_error;

// SPRINT IMPROVEMENTS FOR STABILITY - pensar aqui no PRAGMA integrity_check; , mas como isto implica mexer no codigo para escrever no log ...
// SPRINT IMPROVEMENTS FOR STABILITY - ...  fazer depois de estabilizar a versão 1
const DAILY_PROCEDURE: &str = r"VACUUM;
                                PRAGMA analysis_limit=1000;
                                PRAGMA optimize;
                                PRAGMA wal_checkpoint(TRUNCATE);
                                PRAGMA shrink_memory;";

const LARGE_PROCEDURE: &str = r"
    ATTACH DATABASE ?1 as dw;

    BEGIN     
    INSERT INTO db.aux_mig(min_daily_avg,min_daily,min_cycle,min_sector); 
    SELECT max(a.timestamp),Max(b.timestamp),Max(c.current_run),Max(d.current_run)
            from dw.daily_measure_avg as a,dw.daily_measure as b,dw.watered_cycle as c,dw.watered_sector as d;

    delete from dw.sector;
    delete from dw.sensor;
    insert into dw.sector select * from db.sector;
    insert into dw.sensor select * from db.sensor;

    insert into dw.daily_measure_avg select * from db.daily_measure_avg where timestamp<=?2 and timestamp>(select min_daily_avg from aux_mig);
    insert into dw.daily_measure select * from db.daily_measure where timestamp<=?2 and timestamp>(select min_daily from aux_mig);
    insert into dw.watered_cycle select * from db.watered_cycle where current_run<=?2 and current_run>(select min_cycle from aux_mig);
    insert into dw.watered_sector select * from db.watered_sector where current_run<=?2 and current_run>(select min_sector from aux_mig);

    delete from db.daily_measure_avg where timestamp<=?2;
    delete from db.daily_measure where timestamp<=?2;
    delete from db.watered_cycle where current_run<=?2;
    delete from db.watered_sector where current_run<=?2;

    COMMIT;
    DETACH DATABASE dw;

    VACUUM;
    PRAGMA analysis_limit=1000;
    PRAGMA optimize;
    PRAGMA wal_checkpoint(TRUNCATE);
    PRAGMA shrink_memory;";

#[inline]
pub fn daily_maintenance(_dummy: CtrlTime) -> SimpleResult {
    let t0 = Instant::now();
    let db = LightPersist::new();
    let conn = db.get_conn();
    let mut stmt = conn.prepare_cached(DAILY_PROCEDURE).unwrap();
    let res = stmt.raw_execute();
    #[rustfmt::skip]
    if let Err(e) = res { log_error!(build_error(&e)) };
    debug!("Daily_maintenance ran for {} ", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    Ok(())
}

/// move to backup info > 20 days of age, that its just dead weight in the operational bd (but useful for future stats)
/// to keep "operacional" db lean
#[inline]
pub fn backup_and_maintenance(up_to_date: CtrlTime) -> SimpleResult {
    let t0 = Instant::now();
    let db = LightPersist::new();
    let dw_file = db.config.dw_name.clone();

    let conn = db.get_conn();
    let mut stmt = conn.prepare_cached(LARGE_PROCEDURE).unwrap();

    let _ = stmt.raw_bind_parameter(1, dw_file);
    let _ = stmt.raw_bind_parameter(2, up_to_date.ux_ts());

    let res = stmt.raw_execute();
    #[rustfmt::skip]
    if let Err(e) = res { log_error!(build_error(&e)) };
    info!("Backup_and_maintenance ran for {} ", elapsed_dyn(t0.elapsed().as_nanos() as u64));
    Ok(())
}
