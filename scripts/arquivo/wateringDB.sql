SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0;
SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0;
SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='TRADITIONAL';

DROP SCHEMA IF EXISTS watering;
CREATE SCHEMA watering;
USE watering;

DROP TABLE if EXISTS sector  ;
CREATE TABLE sector (
	id	tinyint not NULL,
	Description	varchar(60) NOT NULL,
	Week_ACC float NOT NULL,
	Precipitation float NOT NULL,
	Debit float NOT NULL,
  	last_watered_in int NOT NULL,
  	enabled int NOT NULL,
	duration float NOT NULL,
	PRIMARY KEY (id) );

DELETE FROM Sector;
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (0, 'Zona Sobreiro', 0.0, 6.985, 6.5, 0, 1, 20);
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (1, 'Zona Nogueira', 0.0, 6.985, 8.0, 0, 1, 20);
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (2, 'Zona Deck Sala', 0.0, 6.985, 4.0, 0, 1, 20);
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (3, 'Zona Amoreira', 0.0, 6.985, 5.0, 0, 1, 20);
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (4, 'Zona Traseiras', 0.0, 3.810, 5.0, 0, 1, 20);
INSERT INTO Sector (id, Description, Week_ACC, Precipitation, Debit, last_watered_in, enabled, duration) 
VALUES (5, 'Zona Norte', 0.0, 6.985, 5.0, 0, 1, 20);


DROP TABLE IF EXISTS Sensor;
CREATE TABLE Sensor (
	id	tinyint NOT NULL,
	Description	varchar(60) NOT NULL,
	UnitShort	varchar(4) NOT NULL,
	PRIMARY KEY(id)
);

DELETE FROM Sensor;
INSERT INTO Sensor (id, Description, UnitShort) VALUES (0, 'Rain', 'mm');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (1, 'Temperature', 'ºC');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (2, 'Max Temperature', 'ºC');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (3, 'Min Temperature', 'ºC');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (4, 'Humidity', '%');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (5, 'Wind Speed', 'km/h');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (6, 'Wind Bearing', 'º');
INSERT INTO Sensor (id, Description, UnitShort) VALUES (7, 'EvapoTranspiration', 'mm');


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

DROP TABLE IF EXISTS ScheduledCycle;
CREATE TABLE ScheduledCycle (
	start_ts int NOT NULL,
	status varchar(15)  NOT NULL,
	start_sunrise_index	tinyint NOT NULL,
	start_sunset_index tinyint NOT NULL,
	repeats	varchar(15) NOT NULL,
	repeat_spec_wd varchar(30),
	repeat_every_qty tinyint NOT NULL,
	repeat_every_unit varchar(15),
	repeat_stop_after varchar(15),
	repeat_stop_retries	tinyint NOT NULL,
	repeat_stop_date int NOT NULL,
	PRIMARY KEY(start_ts)
);

DROP TABLE IF EXISTS ScheduledSector;
CREATE TABLE ScheduledSector (
	idSector tinyint NOT NULL,
	cycle_start_ts int NOT NULL,
	Duration float NOT NULL,
	Status varchar(15) NOT NULL,
	start_ts int NOT NULL,
	end_ts int NOT NULL,
	start_sunrise_index	tinyint NOT NULL,
	start_sunset_index tinyint NOT NULL,
	repeats	varchar(15) NOT NULL,
	repeat_spec_wd	varchar(30),
	repeat_every_qty tinyint NOT NULL,
	repeat_every_unit varchar(15),
	repeat_stop_after varchar(15),
	repeat_stop_retries	tinyint NOT NULL,
	repeat_stop_date int NOT NULL,
	PRIMARY KEY(cycle_start_ts, idSector)
);