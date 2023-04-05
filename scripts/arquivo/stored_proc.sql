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