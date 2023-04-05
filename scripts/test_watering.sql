
select id, cycle_type, name, status, datetime(start, 'unixepoch') as start_d, datetime(end, 'unixepoch') as end_d from watered_cycle order by start_d;
select id_sector, minutes_to_water_acc, status, name, datetime(start, 'unixepoch') as start_d, datetime(end, 'unixepoch') as end_d from watered_sector 
left join sector on watered_sector.id_sector=sector.id where id_ciclo=? and skipped=0 order by start_d;

select *, datetime(start, 'unixepoch') as start_d, datetime(end, 'unixepoch') as end_d, datetime(last_start, 'unixepoch') as last_start_d from watered_sector;

select id_ciclo,cycle_type,name,watered_cycle.status,datetime(start, 'unixepoch'),datetime(end, 'unixepoch') from watered_cycle 
    join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id where start>=0 order by start desc;
select id_sector,minutes_to_water_acc,status,name,datetime(start, 'unixepoch'),datetime(end, 'unixepoch') from watered_sector
    left join sector on watered_sector.id_sector=sector.id where id_ciclo=119244 and skipped=0 order by start,id_ciclo,current_run;

update watered_cycle set start = start-86400;

select * from scheduled_cycle;
select * from watered_cycle;
select * from watered_sector;


ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
select * from pre_prod.scheduled_cycle;

ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;
select * from dev.scheduled_cycle;

select id_sector,minutes_to_water_acc,status,name,start,end from watered_sector 
    left join sector on watered_sector.id_sector=sector.id where id_ciclo=119244 and current_run=0 and skipped=0 order by start,id_ciclo,current_run;

delete from scheduled_cycle where sim = 1;

ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;
select * from dev.sector;

ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
select * from pre_prod.sector;

delete from watered_cycle;
INSERT INTO watered_cycle(status,start,end,id_ciclo,current_run)VALUES(4,1674650000,1674657200, 119244,0);
INSERT INTO watered_cycle(status,start,end,id_ciclo,current_run)VALUES(4,1674664200,1674671400, 119244,1);
INSERT INTO watered_cycle(status,start,end,id_ciclo,current_run)VALUES(4,1674685400,1674692600, 119244,2);

delete from watered_sector;
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(0,119244,0,20,20,0,4,1674650000,1674651200);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(1,119244,0,20,20,0,4,1674651200,1674652400);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(2,119244,0,20,20,0,4,1674652400,1674653600);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(3,119244,0,20,20,0,4,1674653600,1674654800);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(4,119244,0,20,20,0,4,1674654800,1674656000);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(5,119244,0,20,20,0,4,1674656000,1674657200);

INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(0,119244,1,20,20,0,4,1674664200,1674665400);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(1,119244,1,20,20,0,4,1674665400,1674666600);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(2,119244,1,20,20,0,4,1674666600,1674667800);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(3,119244,1,20,20,0,4,1674667800,1674669000);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(4,119244,1,20,20,0,4,1674669000,1674670200);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(5,119244,1,20,20,0,4,1674670200,1674671400);

INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(0,119244,2,20,20,0,4,1674685400,1674686200);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(1,119244,2,20,20,0,4,1674686200,1674687400);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(2,119244,2,20,20,0,4,1674687400,1674688600);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(3,119244,2,20,20,0,4,1674688600,1674689800);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(4,119244,2,20,20,0,4,1674689800,1674690200);
INSERT INTO watered_sector(id_sector,id_ciclo,current_run,minutes_to_water_tgt,minutes_to_water_acc,skipped,status,start,end)
VALUES(5,119244,2,20,20,0,4,1674690200,1674691400);


SELECT sim, id_ciclo,
			DATE_add("1970-1-1", interval start_ts second) as start_ts,
            DATE_add("1970-1-1", interval last_run_ts second) as last_run_ts, status, start_sunrise_index, start_sunset_index, repeat_kind,
            repeat_spec_wd, repeat_every_qty, repeat_every_unit, stop_condition, stop_retries, 
            DATE_add("1970-1-1", interval stop_date_ts second) as stop_date_ts, 
            retries_count, 
            DATE_add("1970-1-1", interval last_change second) as last_change, op FROM watering.scheduledcycle;
SELECT * FROM watering.scheduledcycle where status <> "waiting";
update watering.scheduledcycle set status = "waiting", last_run_ts = start_ts where status <> "waiting";

SELECT * FROM watered_cycle;

select * from scheduled_cycle;
select id_ciclo,watered_cycle.current_run,watered_cycle.status,start,end from watered_cycle 

select id_ciclo,cycle_type,watered_cycle.current_run,name,watered_cycle.status,start,end from watered_cycle 
join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id where start>=0 order by start desc;

select id_ciclo, current_run, max(start) as max from watered_cycle;
id_ciclo	current_run	max
106828	    29	    1677823200

select id_ciclo,cycle_type,watered_cycle.current_run,name,watered_cycle.status,max(start) as start,end from watered_cycle 
join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id ;

select id_ciclo,cycle_type,name,watered_cycle.status,datetime(start, 'unixepoch'),datetime(end, 'unixepoch') from watered_cycle 
    join scheduled_cycle on watered_cycle.id_ciclo=scheduled_cycle.id where start>=0 order by start desc;

select id_sector,minutes_to_water_acc,status,name,datetime(start, 'unixepoch'),datetime(end, 'unixepoch') from watered_sector
    left join sector on watered_sector.id_sector=sector.id where id_ciclo=119244 and skipped=0 order by start,id_ciclo,current_run;
