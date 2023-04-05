CREATE DATABASE `watering` ;
use `watering`;

drop table ref_ids;
CREATE TABLE `ref_ids` (
  `id` varchar(10) NOT NULL,
  `next_num` int(11) NOT NULL,
  PRIMARY KEY (`id`)
)

insert into ref_ids (id, next_num) values ("CYCLE", 0);
insert into ref_ids (id, next_num) values ("SECTOR", 0);
insert into ref_ids (id, next_num) values ("SENSOR", 0);


CREATE TABLE `mods_data` (
  `module` varchar(10) NOT NULL,
  `param` varchar(20) NOT NULL,
  `float` float DEFAULT NULL,
  `int` int(11) DEFAULT NULL,
  `bool` tinyint(4) DEFAULT NULL,
  `string` varchar(45) DEFAULT NULL,
  PRIMARY KEY (`module`,`param`)
)

CREATE TABLE `dailyaveragemeasure` (
  `idSensor` tinyint(4) NOT NULL,
  `timestamp` int(11) NOT NULL,
  `value` float NOT NULL,
  PRIMARY KEY (`idSensor`,`timestamp`)
)

CREATE TABLE `dailymeasures` (
  `idSensor` tinyint(4) NOT NULL,
  `timestamp` int(11) NOT NULL,
  `value` float NOT NULL,
  PRIMARY KEY (`timestamp`,`idSensor`)
) ;

drop table scheduledcycle;

CREATE TABLE `scheduledcycle` (
  `id` int(11) unsigned NOT NULL,
  `sim` tinyint(1) NOT NULL DEFAULT '0',
  `name` varchar(17) NOT NULL,
  `status` varchar(15) NOT NULL,
  `current_run` int(10) unsigned NOT NULL,
  `last_run_ts` int(11) unsigned NOT NULL,
  `op` char(1) NOT NULL DEFAULT 'I',
  `last_change` int(11) NOT NULL,
  `start_ts` int(11) unsigned NOT NULL,
  `start_sunrise_index` tinyint(1) NOT NULL,
  `start_sunset_index` tinyint(1) NOT NULL,
  `repeat_kind` varchar(15) NOT NULL,
  `repeat_spec_wd` varchar(30) DEFAULT '',
  `repeat_every_qty` tinyint(4) NOT NULL,
  `repeat_every_unit` varchar(15) DEFAULT '',
  `stop_condition` varchar(15) DEFAULT '',
  `stop_retries` tinyint(1) NOT NULL,
  `stop_date_ts` int(11) unsigned NOT NULL,
  `retries_count` int(11) NOT NULL,
  PRIMARY KEY (`id`,`sim`),
  UNIQUE KEY `id_UNIQUE` (`id`)
)

drop if exists table `sector`;
CREATE TABLE `sector` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `Week_ACC` float NOT NULL,
  `Precipitation` float NOT NULL,
  `Debit` float NOT NULL,
  `last_watered_in` int(11) NOT NULL,
  `enabled` int(11) NOT NULL,
  `max_duration` float NOT NULL,
  #`status` varchar(15) DEFAULT NULL,
  #`start` int(11) DEFAULT '0',
  #`end` int(11) DEFAULT '0',
  `name` varchar(10) DEFAULT NULL,
  `last_change` int(11) DEFAULT '0',
  `op` varchar(1) DEFAULT 'I',
  `water_level` float DEFAULT '0',
  # `minutes_to_water` float DEFAULT '0',
  PRIMARY KEY (`id`)
) 

DELIMITER ;

DELETE FROM sector;
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (0, 'Zona Sobreiro', 0.0, 6.985, 6.5, 0, 1, 30, 'Sobreiro', 0, 'I', 0);
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (1, 'Zona Nogueira', 0.0, 6.985, 8.0, 0, 1, 30, 'Nogueira', 0, 'I', 0);
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (2, 'Zona Deck Sala', 0.0, 6.985, 4.0, 0, 1, 30, 'Deck Sala', 0, 'I', 0);
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (3, 'Zona Amoreira', 0.0, 6.985, 5.0, 0, 1, 30, 'Amoreira', 0, 'I', 0);
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (4, 'Zona Traseiras', 0.0, 3.810, 5.0, 0, 1, 30, 'Traseiras', 0, 'I', 0);
INSERT INTO sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, max_duration, name, last_change, op, water_level)
VALUES (5, 'Zona Norte', 0.0, 6.985, 5.0, 0, 1, 30, 'Norte', 0, 'I', 0);
update ref_ids set next_num = 6 where id = "SECTOR";

CREATE TABLE `sensor` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `UnitShort` varchar(4) NOT NULL,
  `last_change` int(11) DEFAULT NULL,
  `op` varchar(1) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ;
CREATE TABLE `wateredcycle` (
  `id_ciclo` int(11) NOT NULL,
  `current_run` int(11) NOT NULL,
  `Status` varchar(15) NOT NULL,
  `start` int(11) NOT NULL,
  `end` int(11) NOT NULL,
  PRIMARY KEY (`id_ciclo`,`current_run`),
  KEY `status` (`Status`)
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

drop table `wateredsector`;
CREATE TABLE `wateredsector` (
  `idSector` tinyint(4) NOT NULL,
  `id_ciclo` int(11) NOT NULL,
  `current_run` int(11) NOT NULL,
  `minutes_to_water_tgt` float NOT NULL,
  `minutes_to_water_acc` float NOT NULL,
  `skipped` int(11) NOT NULL,
  `Status` varchar(15) NOT NULL,
  `start` int(11) NOT NULL,
  `end` int(11) NOT NULL,
  PRIMARY KEY (`idSector`,`id_ciclo`,`current_run`),
  KEY `status` (`Status`)
) 

USE watering;
DROP PROCEDURE IF EXISTS `move_data_to_dw`;
DELIMITER $$
CREATE DEFINER=`root`@`localhost` PROCEDURE `move_data_to_dw`(IN ts int, ts_str char(8))
BEGIN
	DECLARE min_date char(8) ;
    DECLARE min_ts int(11);
    
	delete from wateringdw.sector;
    delete from wateringdw.sensor;
    
    insert into wateringdw.sector select * from watering.sector;
    insert into wateringdw.sensor select * from watering.sensor;
    
	select max(yyyymmdd) into min_date from wateringdw.dailyaveragemeasure;
    insert into wateringdw.dailyaveragemeasure select * from watering.dailyaveragemeasure where yyyymmdd <= ts_str and yyyymmdd > min_date;
    
	SELECT Max(timestamp) into min_ts from wateringdw.dailymeasures;
    insert into wateringdw.dailymeasures select * from watering.dailymeasures where timestamp <= ts and timestamp > min_ts;
    
	SELECT Max(cycle_timestamp) into min_ts from wateringdw.wateredcycle;
    insert into wateringdw.wateredcycle select * from watering.wateredcycle where current_run <= ts and current_run > min_ts;
    
	SELECT Max(cycle_timestamp) into min_ts from wateringdw.wateredsector;
    insert into wateringdw.wateredsector select * from watering.wateredsector where current_run <= ts  and current_run > min_ts;
    
    delete from watering.dailyaveragemeasure where yyyymmdd <= ts_str;
    delete from watering.dailymeasures where timestamp <= ts;
    delete from watering.wateredcycle where current_run <= ts;
    delete from watering.wateredsector where current_run <= ts;
    
END$$

DELETE FROM sensor;
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (0, 'Rain', 'mm', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (1, 'Temperature', 'ºC', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (2, 'Max Temperature', 'ºC', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (3, 'Min Temperature', 'ºC', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (4, 'Humidity', '%', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (5, 'Wind Speed', 'km/h', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (6, 'Wind Bearing', 'º', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (7, 'EvapoTranspiration', 'mm', 0, 'I');
INSERT INTO sensor (id, Description, UnitShort, last_change, op) VALUES (8, 'Pressure', 'hpa', 0, 'I');

update ref_ids set next_num = 9 where id = "SENSOR";

CREATE DATABASE `wateringdw` ;
use `wateringdw`;

CREATE TABLE `dailyaveragemeasure` (
  `idSensor` tinyint(4) NOT NULL,
  `timestamp` int(11) NOT NULL,
  `value` float NOT NULL,
  PRIMARY KEY (`idSensor`,`timestamp`)
)


CREATE TABLE `dailymeasures` (
  `idSensor` tinyint(4) NOT NULL,
  `timestamp` int(11) NOT NULL,
  `value` float NOT NULL,
  PRIMARY KEY (`timestamp`,`idSensor`)
) ;
CREATE TABLE `scheduledcycle` (
  `id` int(11) unsigned NOT NULL,
  `sim` tinyint(1) NOT NULL DEFAULT '0',
  `name` varchar(17) NOT NULL,
  `status` varchar(15) NOT NULL,
  `start_ts` int(11) NOT NULL,
  `current_run` int(11) DEFAULT NULL,
  `last_run_ts` int(11) NOT NULL,
  `op` char(1) NOT NULL DEFAULT 'I',
  `last_change` int(11) NOT NULL,
  `start_sunrise_index` tinyint(1) NOT NULL,
  `start_sunset_index` tinyint(1) NOT NULL,
  `repeat_kind` varchar(15) NOT NULL,
  `repeat_spec_wd` varchar(30) DEFAULT '',
  `repeat_every_qty` tinyint(4) NOT NULL,
  `repeat_every_unit` varchar(15) DEFAULT '',
  `stop_condition` varchar(15) DEFAULT '',
  `stop_retries` tinyint(1) NOT NULL,
  `stop_date_ts` int(11) NOT NULL,
  `retries_count` int(11) NOT NULL,
  PRIMARY KEY (`id`,`sim`),
  UNIQUE KEY `id_UNIQUE` (`id`)
) 

drop table `sector`;
CREATE TABLE `sector` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `Week_ACC` float NOT NULL,
  `Precipitation` float NOT NULL,
  `Debit` float NOT NULL,
  `last_watered_in` int(11) NOT NULL,
  `enabled` int(11) NOT NULL,
  `max_duration` float NOT NULL,
  #`status` varchar(15) DEFAULT NULL,
  #`start` int(11) DEFAULT '0',
  #`end` int(11) DEFAULT '0',
  `name` varchar(10) DEFAULT NULL,
  `last_change` int(11) DEFAULT '0',
  `op` varchar(1) DEFAULT 'I',
  `water_level` float DEFAULT '0',
  # `minutes_to_water` float DEFAULT '0',
  PRIMARY KEY (`id`)
) 


CREATE TABLE `sensor` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `UnitShort` varchar(4) NOT NULL,
  `last_change` int(11) DEFAULT NULL,
  `op` varchar(1) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ;
CREATE TABLE `wateredcycle` (
  `id_ciclo` int(11) NOT NULL,
  `current_run` int(11) NOT NULL,
  `Status` varchar(15) NOT NULL,
  `start` int(11) NOT NULL,
  `end` int(11) NOT NULL,
  PRIMARY KEY (`id_ciclo`,`current_run`),
  KEY `status` (`Status`)
) 

# drop table `wateredsector`;
CREATE TABLE `wateredsector` (
  `idSector` tinyint(4) NOT NULL,
  `id_ciclo` int(11) NOT NULL,
  `current_run` int(11) NOT NULL,
  `minutes_to_water_tgt` float NOT NULL,
  `minutes_to_water_acc` float NOT NULL,
  `skipped` int(11) NOT NULL,
  `Status` varchar(15) NOT NULL,
  `start` int(11) NOT NULL,
  `end` int(11) NOT NULL,
  PRIMARY KEY (`idSector`,`id_ciclo`,`current_run`),
  KEY `status` (`Status`)
)


