SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0;
SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0;
SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='TRADITIONAL';

DROP SCHEMA IF EXISTS wateringDW;
CREATE SCHEMA wateringDW;
USE wateringDW;

CREATE TABLE `sync_data` (
  `cli_id` varchar(50) NOT NULL,
  `last_update_ts` int(11) NOT NULL,
  `sync_type` varchar(1) NOT NULL DEFAULT 'F',
  PRIMARY KEY (`cli_id`)
) ;

DROP TABLE if EXISTS sector  ;
CREATE TABLE `sector` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `Week_ACC` float NOT NULL,
  `Precipitation` float NOT NULL,
  `Debit` float NOT NULL,
  `last_watered_in` int(11) NOT NULL,
  `enabled` int(11) NOT NULL,
  `duration` float NOT NULL,
  `real_duration` int(11) DEFAULT NULL,
  `status` varchar(15) DEFAULT NULL,
  `Start_timestamp` int(11) DEFAULT NULL,
  `End_timestamp` int(11) DEFAULT NULL,
  `short_description` varchar(10) DEFAULT NULL,
  `last_change` int(11) DEFAULT NULL,
  `op` varchar(1) DEFAULT NULL,
  PRIMARY KEY (`id`)
);


DROP TABLE IF EXISTS Sensor;
CREATE TABLE `sensor` (
  `id` tinyint(4) NOT NULL,
  `Description` varchar(60) NOT NULL,
  `UnitShort` varchar(4) NOT NULL,
  `last_change` int(11) DEFAULT NULL,
  `op` varchar(1) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ;

DROP TABLE IF EXISTS DailyAverageMeasures ;
CREATE TABLE DailyAverageMeasure (
	idSensor tinyint NOT NULL,
	yyyymmdd char(8) NOT NULL,
	value float NOT NULL,
	PRIMARY KEY(idSensor,yyyymmdd)
);

DROP TABLE IF EXISTS DailyMeasures;
CREATE TABLE DailyMeasures (
	idSensor tinyint NOT NULL,
	timestamp int not NULL,
	value float NOT NULL,
	PRIMARY KEY(timestamp,idSensor)
);


DROP TABLE IF EXISTS WateredCycle;
CREATE TABLE WateredCycle (
	cycle_timestamp	int NOT NULL,
	Status varchar(15) NOT NULL,
	Start_timestamp	int NOT NULL,
	End_timestamp int NOT NULL,
	PRIMARY KEY(cycle_timestamp)
);

DROP TABLE IF EXISTS WateredSector;
CREATE TABLE WateredSector (
	idSector tinyint NOT NULL,
	cycle_timestamp	int NOT NULL,
	Duration float NOT NULL,
	Status	varchar(15) NOT NULL,
	Start_timestamp	int NOT NULL,
	End_timestamp int NOT NULL,
	PRIMARY KEY(idSector,cycle_timestamp)
);

DROP TABLE IF EXISTS scheduledcycle;
CREATE TABLE `scheduledcycle` (
  `sim` tinyint(1) NOT NULL DEFAULT '0',
  `nome` char(15) NOT NULL,
  `start_ts` int(11) NOT NULL,
  `last_run_ts` int(11) NOT NULL,
  `status` varchar(15) NOT NULL,
  `start_sunrise_index` tinyint(1) NOT NULL,
  `start_sunset_index` tinyint(1) NOT NULL,
  `repeats` varchar(15) NOT NULL,
  `repeat_spec_wd` varchar(30) DEFAULT '',
  `repeat_every_qty` tinyint(4) NOT NULL,
  `repeat_every_unit` varchar(15) DEFAULT '',
  `repeat_stop_after` varchar(15) DEFAULT '',
  `repeat_stop_retries` tinyint(1) NOT NULL,
  `repeat_stop_date` int(11) NOT NULL,
  `retries_count` int(11) NOT NULL,
  `last_change` int(11) NOT NULL,
  `op` char(1) NOT NULL DEFAULT 'I',
  `id` int(11) NOT NULL,
  PRIMARY KEY (`id`,`sim`)
) ;




commit