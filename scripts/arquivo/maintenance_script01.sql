CREATE PROCEDURE `move_data_to_dw` (IN ts int, ts_str char(8))
BEGIN

    insert into wateringdw.dailyaveragemeasures select * from watering.dailyaveragemeasures where yyyymmdd <= ts_str;
    insert into wateringdw.DailyMeasures select * from watering.DailyMeasures  where timestamp <= ts;
    insert into wateringdw.WateredCycle select * from watering.WateredCycle  where timestamp <= ts;
    insert into wateringdw.WateredSector select * from watering.WateredSector  where timestamp <= ts;

    delete from watering.dailyaveragemeasures where yyyymmdd <= ts_str;
    delete from watering.DailyMeasures where timestamp <= ts;
    delete from watering.WateredCycle where timestamp <= ts;
    delete from watering.WateredSector where timestamp <= ts;
    
END
