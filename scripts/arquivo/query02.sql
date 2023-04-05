use watering;
CALL `move_data_to_dw`(1576540800, '20191217');

    
	delete from wateringdw.sector;
    delete from wateringdw.sensor;
    
    insert into wateringdw.sector select * from watering.sector;
    insert into wateringdw.sensor select * from watering.sensor;
    
	select max(yyyymmdd) from wateringdw.dailyaveragemeasure;
    insert into wateringdw.dailyaveragemeasure select * from watering.dailyaveragemeasure where yyyymmdd <= '20191217'and yyyymmdd > '20191207';
    
	SELECT MIN(timestamp) into min_ts from wateringdw.dailymeasures;
    insert into wateringdw.dailymeasures select * from watering.dailymeasures where timestamp <= 1576540800 and timestamp > min_ts;
    
	SELECT MIN(cycle_timestamp) into min_ts from wateringdw.wateredcycle;
    insert into wateringdw.wateredcycle select * from watering.wateredcycle where cycle_timestamp <= 1576540800 and cycle_timestamp > min_ts;
    
	SELECT MIN(cycle_timestamp) into min_ts from wateringdw.wateredsector;
    insert into wateringdw.wateredsector select * from watering.wateredsector where cycle_timestamp <= 1576540800;
    

    select min(yyyymmdd) from wateringdw.dailyaveragemeasure;
    SELECT MIN(timestamp) from wateringdw.dailymeasures;
DELIMITER $$
create procedure a()
BEGIN
	DECLARE min_date char(8) ;
    DECLARE min_ts int(11);
    
    select min(yyyymmdd) into min_date from wateringdw.dailyaveragemeasure;
	SELECT MIN(timestamp) into min_ts from wateringdw.dailymeasures;
    select * from watering.dailyaveragemeasure where yyyymmdd <= '20191217' AND yyyymmdd > '20191128';
    select * from watering.wateredsector  where cycle_timestamp <= 1576540800 and cycle_timestamp > 1574899140;
END$$