// DESIGN INFO
// o sqllite não sendo client server, não tem os temas de várias round trips á bd
// é por isso que nesta implementação temos várias chamadas consecutivas á bd.
// É porque o sqlite não têm o problema de "N+1 query".
// Funciona tudo no mesmo processo e no essencial são chamadas a funções sempre dentro do mesmo processo (sqllite dll)
// existem sistemas com mais de 200 chamadas/queries e não é tema.  Aqui tenho de 2 a 4 (eventualmente um pouco mais em algum processo)
// pelo que não é tema, e o código fica mais simples - e neste caso mais simple de manter no sql e passagem de parametros ás queries
//
// Tomei ainda a decisão de colocar muitos ids hard coded.
// é mais eficiente que usar strings (sim, sei que marginalmente...), mas sendo uma aplicação só "para mim", assumi este risco
// de manutenção.  Na prática acho que o tema da manutenção seria igual, com isto ou sem isto, i.e., daqui a 6 meses não me lembrarei nem de uma
// coisa nem de outra.
// Pontualmente na BD temnho tabelas de referencia para fazer uns joins se quiser ou recisar de ler estas colunas com estes valores "hard coded"

use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::irrigation::{cycle::*, cycle_run::*, sector::*, sector_run::*, wtr_engine::*, wtr_history::*};
use crate::{app_time::ctrl_time::*, log_error, logger::*};
use ctrl_prelude::{domain_types::*, error::*};

pub trait DBModelRega<'a>: DB {
    const FIND_SCHEDULE_SELECT: &'a str = "select id,name,start_ts,status,last_run,start_sunrise_index,start_sunset_index,repeat_kind,\
                                            repeat_spec_wd,repeat_every_qty,repeat_every_unit,stop_condition,stop_retries,stop_date_ts,\
                                            retries_count,last_change,op,cycle_type \
                                            from scheduled_cycle where start_ts>=? and start_ts<=?;";

    const WATERED_SECTOR_INSERT: &'a str = "INSERT INTO watered_sector(minutes_to_water_tgt,minutes_to_water_acc,skipped,status,\
                                            start,end,last_start,id_ciclo,current_run,id_sector)VALUES(?,?,?,?,?,?,?,?,?,?);";

    const WATERED_SECTOR_WTR_UPDATE: &'a str = "update watered_sector \
                                            set minutes_to_water_tgt=?,minutes_to_water_acc=?,skipped=?,status=?,start=?,end=?,last_start=? \
                                            where id_ciclo=? and current_run=? and id_sector=?;";

    const SCHEDULE_CYCLE_DELETE: &'a str = "delete from scheduled_cycle where id=?;";

    const GET_ALL_SCHEDULE: &'a str = "select id,name,status,current_run,start_ts,last_run,start_sunrise_index,start_sunset_index,\
                                        repeat_kind,repeat_spec_wd,repeat_every_qty,repeat_every_unit,stop_condition,stop_retries,\
                                        stop_date_ts,retries_count,last_change,op,cycle_type from scheduled_cycle order by cycle_type;";

    const GET_ALL_SECTOR: &'a str = "select id,description,deficit,percolation,debit,last_watered_in,enabled,\
                                    max_duration,name,last_change,op, device_id from sector order by id;";

    const SCHEDULE_INSERT: &'a str = "INSERT INTO scheduled_cycle(name,current_run,status,last_run,start_sunrise_index,\
                                    start_sunset_index,start_ts,repeat_kind,repeat_spec_wd,repeat_every_qty,\
                                    repeat_every_unit,stop_condition,stop_retries,stop_date_ts,retries_count,\
                                    last_change,op,cycle_type)VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?);";

    const WATERED_CYCLE_INSERT: &'a str = "INSERT INTO watered_cycle(status,start,end,id_ciclo,current_run)VALUES(?,?,?,?,?)";

    const WATERED_CYCLE_UPDATE: &'a str = "update watered_cycle SET status=?,start=?,end=? where id_ciclo=? and current_run=?;";

    const SECTOR_UPDATE: &'a str = "update sector set description=?,deficit=?,percolation=?,debit=?,last_watered_in=?,enabled=?,\
                                max_duration=?,name=?,last_change=?,op=? where id=?;";

    const SCHEDULE_CYCLE_CLIENT_UPDATE: &'a str = "update scheduled_cycle set name=?, start_sunrise_index=?,start_sunset_index=?,\
                                    start_ts=?,repeat_kind=?,repeat_spec_wd=?,repeat_every_qty=?,repeat_every_unit=?,\
                                    stop_condition=?,stop_retries=?,stop_date_ts=?,retries_count=?,last_change=?,op=? where id=?;";

    const SCHEDULE_CYCLE_SERVER_UPDATE: &'a str = "update scheduled_cycle set current_run=?,status=?,last_run=?,start_ts=?,last_change=? where id=?;";

    /// ( 0, 'WAITING' ); ( 1, 'RUNNING ' ); ( 2, 'SUSPENDED' );( 3, 'NOT EXECUTED' );( 4, 'TERMINATED' ); ( 5, 'ERROR' );
    const RECOVER_SCHEDULED_CYCLE: &'a str = "update scheduled_cycle set status=0,last_run=start_ts where status<>0;";
    /// ( 0, 'WAITING' ); ( 1, 'RUNNING ' ); ( 2, 'SUSPENDED' );( 3, 'NOT EXECUTED' );( 4, 'TERMINATED' ); ( 5, 'ERROR' );
    const RECOVER_WATERED_CYCLE: &'a str = "update watered_cycle SET status=5 where status not in(5,3,4);";
    /// ( 0, 'WAITING' ); ( 1, 'RUNNING ' ); ( 2, 'SUSPENDED' );( 3, 'NOT EXECUTED' );( 4, 'TERMINATED' ); ( 5, 'ERROR' );
    const RECOVER_WATERED_SECTOR: &'a str = "update watered_sector set status=case when status=1 THEN 5 when status=0 THEN 3 END where status=1 or status=0;";

    const GET_WATERED_CYCLES: &'a str = "select id_ciclo,cycle_type,watered_cycle.current_run,name,watered_cycle.status,start,end from watered_cycle \
                                        right join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id where start>=? order by start desc;";
    const GET_LAST_WATERED_CYCLE: &'a str =
        "select id_ciclo,cycle_type,watered_cycle.current_run,name,watered_cycle.status,max(start) as start,end from watered_cycle \
                                        right join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id;";
    const GET_WATERED_SECTORS: &'a str = "select id_sector,minutes_to_water_acc,status,name,start,end from watered_sector \
                                        left join sector on watered_sector.id_sector=sector.id where id_ciclo=? and current_run=? and skipped=0 order by start,id_ciclo,current_run;";

    #[inline]
    fn get_cfg_secs(&self, sectors: &mut SecList) -> Result<(), DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_ALL_SECTOR).unwrap();
        let mut rows = stmt.raw_query();
        while let Some(row) = rows.next()? {
            sectors.push(row.into());
        }
        Ok(())
    }

    #[inline]
    fn find_cycle(&self, start: CtrlTime, end: CtrlTime) -> Result<Option<Cycle>, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::FIND_SCHEDULE_SELECT).unwrap();
        let mut cycle: Option<Cycle> = None;

        _ = stmt.raw_bind_parameter(1, start.ux_ts());
        _ = stmt.raw_bind_parameter(2, end.ux_ts());

        let mut rows = stmt.raw_query();
        if let Some(row) = rows.next()? {
            cycle = Some(row.into());
        }
        Ok(cycle)
    }

    #[inline]
    fn get_all_cycles(&self, cycles: &mut CycleList) -> Result<(), DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_ALL_SCHEDULE).unwrap();

        let mut rows = stmt.raw_query();
        let mut cycle: Cycle;
        while let Some(row) = rows.next()? {
            cycle = row.into();
            cycles.push(cycle);
        }
        Ok(())
    }

    #[inline]
    fn upd_cycle_srvr(&self, cycle: &Cycle) -> SimpleResult {
        let cycle = cycle;
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SCHEDULE_CYCLE_SERVER_UPDATE).unwrap();

        _ = stmt.raw_bind_parameter(1, cycle.run.run_id);
        _ = stmt.raw_bind_parameter(2, cycle.run.status as u8);
        _ = stmt.raw_bind_parameter(3, cycle.last_run.ux_ts());
        _ = stmt.raw_bind_parameter(4, cycle.schedule.start.ux_ts());
        _ = stmt.raw_bind_parameter(5, cycle.last_change.ux_ts());
        _ = stmt.raw_bind_parameter(6, cycle.run.cycle_id);

        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn upd_sec(&self, sector: &Sector) -> SimpleResult {
        let sector = sector;
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SECTOR_UPDATE).unwrap();
        raw_sec_params(&mut stmt, sector);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn upd_cycle_run(&self, cycle_run: &CycleRun) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::WATERED_CYCLE_UPDATE).unwrap();
        cycle_run_params(&mut stmt, cycle_run);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn upd_sec_run(&self, ws: &SectorRun) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::WATERED_SECTOR_WTR_UPDATE).unwrap();
        run_sec_params(&mut stmt, ws);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn ins_cycle_run(&self, cycle_run: &CycleRun) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::WATERED_CYCLE_INSERT).unwrap();
        cycle_run_params(&mut stmt, cycle_run);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn ins_secs_run(&self, ws: &SectorRun) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::WATERED_SECTOR_INSERT).unwrap();
        run_sec_params(&mut stmt, ws);
        _ = self.exec_prep(&mut stmt); // o exec_prep já grava o erro no log
        Ok(())
    }

    #[inline]
    fn upd_secs_batch(&self, sectors: &[Sector]) -> SimpleResult {
        let sectors = sectors;
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SECTOR_UPDATE).unwrap();

        for sector in sectors {
            raw_sec_params(&mut stmt, sector);
            _ = stmt.raw_execute().map_err(|e| log_error!(build_error(&e)));
        }
        Ok(())
    }

    #[inline]
    fn del_cycle_by_id(&self, id: CYCLE_ID) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SCHEDULE_CYCLE_DELETE).unwrap();
        _ = stmt.raw_bind_parameter(1, id);
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn ins_cycle(&self, cycle: &mut Cycle) -> SimpleResult {
        let cycle = cycle;
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::SCHEDULE_INSERT).unwrap();

        _ = stmt.raw_bind_parameter(1, &cycle.name);
        _ = stmt.raw_bind_parameter(2, cycle.run.run_id);
        _ = stmt.raw_bind_parameter(3, cycle.run.status as u8);
        _ = stmt.raw_bind_parameter(4, cycle.last_run.ux_ts());
        _ = stmt.raw_bind_parameter(5, cycle.sunrise_flg);
        _ = stmt.raw_bind_parameter(6, cycle.sunset_flg);
        _ = stmt.raw_bind_parameter(7, cycle.schedule.start.ux_ts());
        _ = stmt.raw_bind_parameter(8, cycle.schedule.repeat_kind as u8);
        _ = stmt.raw_bind_parameter(9, cycle.schedule.repeat_spec_wd);
        _ = stmt.raw_bind_parameter(10, cycle.schedule.repeat_every_qty);
        _ = stmt.raw_bind_parameter(11, cycle.schedule.repeat_every_unit as u8);
        _ = stmt.raw_bind_parameter(12, cycle.schedule.stop_condition as u8);
        _ = stmt.raw_bind_parameter(13, cycle.schedule.stop_retries);
        _ = stmt.raw_bind_parameter(14, cycle.schedule.stop_date_ts.ux_ts());
        _ = stmt.raw_bind_parameter(15, cycle.schedule.retries_count);
        _ = stmt.raw_bind_parameter(16, cycle.last_change.ux_ts());
        _ = stmt.raw_bind_parameter(17, cycle.op.to_str());
        _ = stmt.raw_bind_parameter(18, cycle.cycle_type as u8);

        self.exec_prep(&mut stmt)?;
        cycle.run.cycle_id = conn.last_insert_rowid() as CYCLE_ID;
        Ok(())
    }

    #[inline]
    fn recover_inconsistent_cycles(&self) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::RECOVER_SCHEDULED_CYCLE).unwrap();
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn recover_cycle_run(&self) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::RECOVER_WATERED_CYCLE).unwrap();
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn recover_secs_run(&self) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::RECOVER_WATERED_SECTOR).unwrap();
        self.exec_prep(&mut stmt)
    }

    #[inline]
    fn get_water_history(&self, time: UTC_UNIX_TIME) -> Result<WaterHstry, DBError> {
        let conn = &self.get_conn().conn;
        let ref_time = CtrlTime::from_ux_ts(time).sod_ux_e().sub_days(15);

        // vamos buscar as regas dos ultimos 15 dias
        let mut stmt = conn.prepare_cached(Self::GET_WATERED_CYCLES).unwrap();
        _ = stmt.raw_bind_parameter(1, ref_time.ux_ts());
        let mut rows = stmt.raw_query();

        let mut wtr = WaterHstry::new();

        let mut cycle: CycleHstry;
        while let Some(row) = rows.next()? {
            cycle = row.into();
            wtr.cycles.push(cycle);
        }

        // se não houver linhas, vamos á procura da ultima rega
        if wtr.cycles.is_empty() {
            let mut stmt = conn.prepare_cached(Self::GET_WATERED_CYCLES).unwrap();
            let mut rows = stmt.raw_query();
            while let Some(row) = rows.next()? {
                cycle = row.into();
                wtr.cycles.push(cycle);
            }
        }
        for i in 0..wtr.cycles.len() {
            let mut stmt = conn.prepare_cached(Self::GET_WATERED_SECTORS).unwrap();
            _ = stmt.raw_bind_parameter(1, wtr.cycles[i].cycleid);
            _ = stmt.raw_bind_parameter(2, wtr.cycles[i].current_run);
            let mut rows = stmt.raw_query();
            let mut sector: SectorHstry;
            while let Some(row) = rows.next()? {
                sector = row.into();
                wtr.cycles[i].sectors.push(sector);
            }
        }

        Ok(wtr)
    }
}

#[inline]
pub fn raw_sec_params(stmt: &mut rusqlite::CachedStatement, sector: &Sector) {
    let sector = sector;
    let stmt = stmt;
    _ = stmt.raw_bind_parameter(1, &sector.desc);
    _ = stmt.raw_bind_parameter(2, sector.deficit);
    _ = stmt.raw_bind_parameter(3, sector.percolation);
    _ = stmt.raw_bind_parameter(4, sector.debit);
    _ = stmt.raw_bind_parameter(5, sector.last_watered_in.ux_ts());
    _ = stmt.raw_bind_parameter(6, sector.enabled);
    _ = stmt.raw_bind_parameter(7, sector.max_duration);
    _ = stmt.raw_bind_parameter(8, &sector.name);
    _ = stmt.raw_bind_parameter(9, sector.last_change.ux_ts());
    _ = stmt.raw_bind_parameter(10, sector.op.to_str());
    _ = stmt.raw_bind_parameter(11, sector.id);
}

// const WATERED_SECTOR_INSERT: &'a str = "INSERT INTO watered_sector(minutes_to_water_tgt,minutes_to_water_acc,skipped,status,\
//      start,end,last_start,id_ciclo,current_run,id_sector)VALUES(?,?,?,?,?,?,?,?,?,?);";
// const WATERED_SECTOR_WTR_UPDATE: &'a str = "update watered_sector \
//      set minutes_to_water_tgt=?,minutes_to_water_acc=?,skipped=?,status=?,start=?,end=?,last_start=? \
//      where id_ciclo=? and current_run=? and id_sector=?;";
#[inline]
pub fn run_sec_params(stmt: &mut rusqlite::CachedStatement, ws: &SectorRun) {
    let stmt = stmt;
    _ = stmt.raw_bind_parameter(1, ws.wtr_tgt_min);
    _ = stmt.raw_bind_parameter(2, ws.wtr_acc_min);
    _ = stmt.raw_bind_parameter(3, ws.skipped as u8);
    _ = stmt.raw_bind_parameter(4, ws.status as u8);
    _ = stmt.raw_bind_parameter(5, ws.start.ux_ts());
    _ = stmt.raw_bind_parameter(6, ws.end.ux_ts());
    _ = stmt.raw_bind_parameter(7, ws.last_start.ux_ts());
    _ = stmt.raw_bind_parameter(8, ws.cycle_id);
    _ = stmt.raw_bind_parameter(9, ws.curr_run);
    _ = stmt.raw_bind_parameter(10, ws.sec_id);
}

// "INSERT INTO watered_cycle(status,start,end,id_ciclo,current_run)VALUES(?,?,?,?,?)";
// const WATERED_CYCLE_UPDATE: &'a str = "update watered_cycle SET status=?,start=?,end=? where id_ciclo=? and current_run=?;";
#[inline]
pub fn cycle_run_params(stmt: &mut rusqlite::CachedStatement, cycle_run: &CycleRun) {
    let stmt = stmt;
    _ = stmt.raw_bind_parameter(1, cycle_run.status as u8);
    _ = stmt.raw_bind_parameter(2, cycle_run.start.ux_ts());
    _ = stmt.raw_bind_parameter(3, cycle_run.end.ux_ts());
    _ = stmt.raw_bind_parameter(4, cycle_run.cycle_id);
    _ = stmt.raw_bind_parameter(5, cycle_run.run_id);
}

impl<'a> DBModelRega<'a> for Persist {}
