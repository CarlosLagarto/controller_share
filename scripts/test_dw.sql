-- SQLite
-- ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as db;
BEGIN;

DELETE from db.aux_mig;
        
INSERT INTO db.aux_mig(min_daily_avg,min_daily,min_cycle,min_sector); 
SELECT max(a.timestamp),Max(b.timestamp),Max(c.current_run),Max(d.current_run)
        from dw.daily_measure_avg as a,dw.daily_measure as b,dw.watered_cycle as c,dw.watered_sector as d;

delete from dw.sector;
delete from dw.sensor;
insert into dw.sector select * from db.sector;
insert into dw.sensor select * from db.sensor;

insert into dw.daily_measure_avg select * from db.daily_measure_avg where timestamp<=?1 and timestamp>(select min_daily_avg from aux_mig);
insert into dw.daily_measure select * from db.daily_measure where timestamp<=?1 and timestamp>(select min_daily from aux_mig);
insert into dw.watered_cycle select * from db.watered_cycle where current_run<=?1 and current_run>(select min_cycle from aux_mig);
insert into dw.watered_sector select * from db.watered_sector where current_run<=?1 and current_run>(select min_sector from aux_mig);

delete from db.daily_measure_avg where timestamp<=?1;
delete from db.daily_measure where timestamp<=?1;
delete from db.watered_cycle where current_run<=?1;
delete from db.watered_sector where current_run<=?1;

COMMIT;

VACUUM;
PRAGMA analysis_limit=1000;
PRAGMA optimize;
PRAGMA wal_checkpoint(TRUNCATE);
PRAGMA shrink_memory;
--COMMIT;
----------------------------

ATTACH DATABASE '/home/lagarto/DEV/RUST/controller/src/data/dw.db' as dw;

BEGIN     
INSERT INTO db.aux_mig(min_daily_avg,min_daily,min_cycle,min_sector); 
SELECT max(a.timestamp),Max(b.timestamp),Max(c.current_run),Max(d.current_run)
        from dw.daily_measure_avg as a,dw.daily_measure as b,dw.watered_cycle as c,dw.watered_sector as d;

delete from dw.sector;
delete from dw.sensor;
insert into dw.sector select * from db.sector;
insert into dw.sensor select * from db.sensor;

insert into dw.daily_measure_avg select * from db.daily_measure_avg where timestamp<=?1 and timestamp>(select min_daily_avg from aux_mig);
insert into dw.daily_measure select * from db.daily_measure where timestamp<=?1 and timestamp>(select min_daily from aux_mig);
insert into dw.watered_cycle select * from db.watered_cycle where current_run<=?1 and current_run>(select min_cycle from aux_mig);
insert into dw.watered_sector select * from db.watered_sector where current_run<=?1 and current_run>(select min_sector from aux_mig);

delete from db.daily_measure_avg where timestamp<=?1;
delete from db.daily_measure where timestamp<=?1;
delete from db.watered_cycle where current_run<=?1;
delete from db.watered_sector where current_run<=?1;

COMMIT;
DETACH DATABASE dw;

VACUUM;
PRAGMA analysis_limit=1000;
PRAGMA optimize;
PRAGMA wal_checkpoint(TRUNCATE);
PRAGMA shrink_memory;




-----------------------------
CREATE DEFINER=`root`@`localhost` PROCEDURE `move_data_to_dw`(IN ts int, ts_str char(8))
BEGIN
	
	DECLARE EXIT HANDLER FOR SQLEXCEPTION
	BEGIN
	GET DIAGNOSTICS CONDITION 1 @sqlstate = RETURNED_SQLSTATE, 
	 @errno = MYSQL_ERRNO, @text = MESSAGE_TEXT;
	SET @full_error = CONCAT("ERROR ", @errno, " (", @sqlstate, "): ", @text);
	SELECT @full_error;
	END;

	SET SQL_SAFE_UPDATES = 0;
    
	delete from wateringdw.sector;
    delete from wateringdw.sensor;
    
    insert into wateringdw.sector select * from watering.sector;
    insert into wateringdw.sensor select * from watering.sensor;
    
    insert into wateringdw.dailyaveragemeasure select * from watering.dailyaveragemeasure where yyyymmdd <= ts_str;
    insert into wateringdw.dailymeasures select * from watering.dailymeasures  where timestamp <= ts;
    insert into wateringdw.wateredCycle select * from watering.wateredCycle  where cycle_timestamp <= ts;
    insert into wateringdw.wateredSector select * from watering.wateredSector  where cycle_timestamp <= ts;

	
    delete from watering.dailyaveragemeasure where yyyymmdd <= ts_str;
    delete from watering.dailymeasures where timestamp <= ts;
    delete from watering.wateredcycle where cycle_timestamp <= ts;
    delete from watering.wateredsector where cycle_timestamp <= ts;

END