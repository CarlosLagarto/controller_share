
select * from sensor_data  WHERE timestamp>1667044350 AND timestamp<=1667130750 AND  id_sensor=4 ;

SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,AVG(value) AS avg_value 
        FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id 
        WHERE timestamp>1667044350 AND timestamp<=1667130750 AND  id_sensor=3     
        GROUP BY id_sensor,minutets;


-- SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets, ((atan2(AVG(sin((value)*3.14/180)),AVG(cos((value)*3.14/180)))+360)%360)*180/3.14 as avg_value 
--         FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id 
--             WHERE timestamp>1667044350 AND timestamp<=1667130750 AND  id_sensor=4 
--             GROUP BY id_sensor,minutets ORDER BY id_sensor,minutets

-- SELECT t1.minutets,27785512-t1.minutets AS diff, 
-- COALESCE(CASE WHEN id_sensor=3 THEN AVG(avg_value) END,0) AS val1,
-- COALESCE(CASE WHEN id_sensor=4 THEN degrees((atan2(AVG(sin(radians(avg_value))),AVG(cos(radians(avg_value))))+360)%360) END,0) AS val2 
-- FROM (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,AVG(value) AS avg_value 
--         FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id 
--         WHERE timestamp>1667044350 AND timestamp<=1667130750 AND  id_sensor=3     
--         GROUP BY id_sensor,minutets 
--     UNION 
--         SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets, degrees((atan2(AVG(sin(radians(value))),AVG(cos(radians(value))))+360)%360) as avg_value 
--         FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id 
--             WHERE timestamp>1667044350 AND timestamp<=1667130750 AND  id_sensor=4 
--             GROUP BY id_sensor,minutets ORDER BY id_sensor,minutets) 
-- AS t1 
-- GROUP BY t1.minutets ORDER BY t1.minutets;


select * from scheduled_cycle inner join watered_cycle ON id=watered_cycle.id_ciclo where watered_cycle.start >=1643003549 and scheduled_cycle.sim = 1;

select * from scheduled_cycle inner join watered_cycle ON id=watered_cycle.id_ciclo where scheduled_cycle.sim = 1;

select * from scheduled_cycle ;

select string from mods_data where module = 2 and param = 3 order by module, param ;

update mods_data set string="Manual" where module=2 and param=3;

 DELETE  from db.aux_mig;

--1668585340 E+9
--1668671740

SELECT t1.minutets,27811196-t1.minutets AS diff,
    COALESCE(CASE WHEN id_sensor=3 THEN AVG(avg_value) END,0) AS val1,
    COALESCE(CASE WHEN id_sensor=4 THEN (degrees(atan2(AVG(sin(radians(avg_value))),AVG(cos(radians(avg_value)))))+360)%360 END,0) AS val2 FROM 
        (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            AVG(value) AS avg_value 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=3 
            GROUP BY id_sensor,minutets 
        UNION 
        SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            (degrees(atan2(AVG(sin(radians(value))),AVG(cos(radians(value)))))+360)%360 as avg_value 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=4 
            GROUP BY id_sensor,minutets 
                                ORDER BY id_sensor,minutets) AS t1 GROUP BY t1.id_sensor, t1.minutets ORDER BY t1.minutets;
                                
SELECT t1.minutets,27811196-t1.minutets AS diff,
    COALESCE(CASE WHEN id_sensor=3 THEN AVG(avg_value) END,0) AS val1,
    COALESCE(CASE WHEN id_sensor=4 THEN (degrees(atan2(AVG(sin(radians(avg_value))),AVG(cos(radians(avg_value)))))+360)%360 END,0) AS val2 FROM 
        (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            AVG(value) AS avg_value 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=3 
            GROUP BY id_sensor,minutets 
        UNION 
        SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            (degrees(atan2(AVG(sin(radians(value))),AVG(cos(radians(value)))))+360)%360 as avg_value 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=4 
            GROUP BY id_sensor,minutets 
                                ORDER BY id_sensor,minutets) AS t1 GROUP BY t1.id_sensor, t1.minutets ORDER BY t1.minutets;
                                
explain query plan select T1.minutets, val1, val2 from (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            AVG(value) AS val1 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=3 
            GROUP BY minutets ) T1
         INNER JOIN 
        (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            (degrees(atan2(AVG(sin(radians(value))),AVG(cos(radians(value)))))+360)%360 as val2 
            FROM sensor_data INNER JOIN sensor ON id_sensor=sensor.id WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=4 
            GROUP BY minutets) T2 ON T1.minutets = T2.minutets ;
            
explain query plan select T1.minutets, 27811196-t1.minutets AS diff, val1, val2 from (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            AVG(value) AS val1 
            FROM sensor_data WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=3 
            GROUP BY minutets ) T1
         INNER JOIN 
        (SELECT id_sensor,CAST(timestamp/60 AS INT) AS minutets,
            (degrees(atan2(AVG(sin(radians(value))),AVG(cos(radians(value)))))+360)%360 as val2 
            FROM sensor_data WHERE timestamp>1668585340 AND timestamp<=1668671740 AND  id_sensor=4 
            GROUP BY minutets) T2 ON T1.minutets = T2.minutets order by T1.minutets;
            
                                ORDER BY id_sensor,minutets) AS t1 GROUP BY t1.id_sensor, t1.minutets ORDER BY t1.minutets;

select * from sensor;

select * from metric;

SELECT * from device;

ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;

insert into dev.sensor_data select * from pre_prod.sensor_data where pre_prod.sensor_data.timestamp > 1675987200 and pre_prod.sensor_data.timestamp < 1676073600;

ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
select * from pre_prod.sensor_data where pre_prod.sensor_data.timestamp > 1675987200 and pre_prod.sensor_data.timestamp < 1676073600;

ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;
select * from dev.sensor_data 
where dev.sensor_data.timestamp > 1675987200 and dev.sensor_data.timestamp < 1676073600;

ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
select * from pre_prod.sensor_daily_data;

ATTACH DATABASE "/home/lagarto/pre-prod/db/db.db" as pre_prod;
select distinct id_metric from pre_prod.sensor_daily_data;

ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;
select id_sensor, count(*) as count from dev.sensor_data where dev.sensor_data.timestamp > 1675987200 and dev.sensor_data.timestamp < 1676073600 group by id_sensor;


ATTACH DATABASE "/home/lagarto/DEV/RUST/controller/src/data/db.db" as dev;
select * from dev.sensor_data_daily
 where dev.sensor_data.timestamp > 1675987200 and dev.sensor_data.timestamp < 1676073600 group by id_sensor;

--inicio dia 10 1675987200
-- inicio dia 11 1676073601
select * from sensor_data where timestamp > 1675987200 and timestamp < 1676073600;

--  order by timestamp desc;


select * from ml_model_data;
-- UPDATE aplicado em pre-prod em 2023-01-22
-- 20230122_update.sql

-- UPDATE aplicado em pre-prod em 2023-03-01
-- 20230301_update.sql

-- SCRIPT para atualizar produção - alterações - em curso depois de 2023-03-1