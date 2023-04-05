PRAGMA journal_mode = MEMORY;
PRAGMA synchronous = OFF;
PRAGMA foreign_keys = OFF;
PRAGMA ignore_check_constraints = OFF;
PRAGMA auto_vacuum = NONE;
PRAGMA secure_delete = OFF;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS water_state (
    'id'     INTEGER      NOT NULL,
    'status' TEXT         NOT NULL
    PRIMARY KEY('id')
) WITHOUT ROWID;

DELETE FROM water_state;
INSERT INTO water_state (id,status) VALUES ( 0, 'WAITING' );
INSERT INTO water_state (id,status) VALUES ( 1, 'RUNNING ' );
INSERT INTO water_state (id,status) VALUES ( 2, 'SUSPENDED' );
INSERT INTO water_state (id,status) VALUES ( 3, 'NOT EXECUTED' );
INSERT INTO water_state (id,status) VALUES ( 4, 'TERMINATED' );
INSERT INTO water_state (id,status) VALUES ( 5, 'ERROR' );

-- CREATE TABLE 'ref_ids' (
--     'id'       INTEGER NOT NULL,
--     'desc'   TEXT    NOT NULL,
--     'next_num' INTEGER NOT NULL,
--     PRIMARY KEY ('id')
-- ) WITHOUT ROWID;

-- insert into ref_ids (id, desc, next_num) values (0,	'CYCLE',	0);
-- insert into ref_ids (id, desc, next_num) values (1,	'SECTOR',	6);
-- insert into ref_ids (id, desc, next_num) values (2,	'SENSOR',	9);

CREATE TABLE 'mods_data' (
    'module'    INTEGER NOT NULL,
    'param'     INTEGER NOT NULL,
    'float'     FLOAT   DEFAULT NULL,
    'int'       INTEGER DEFAULT NULL,
    'string'    TEXT    DEFAULT NULL,
    'name'      TEXT,
    'descricao' TEXT,
    PRIMARY KEY ( 'module', 'param' )
) WITHOUT ROWID;

insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,0,0,0,'','CurrentDay','dia corrente no programa')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,1,0,0,'2020-08-13T05:01:45.690565000Z','LiveSince','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,2,0,1597294905,'','LastChange','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,3,0,0,'','LastClientUpdate','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,4,0,5,'','CheckClientInterval','verifica se há alterações a enviar ao cliente a cada X segundos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,5,0,0,'','ShutDown','se =1, foi não controlado')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,6,0,10,'','FileSaveInterval','grava contexto a cada X segundos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,7,0,0,'','StartDateStr','ISO Format "%Y-%m-%dT%H:%M:%S%.fZ" ')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,8,0,0,'','StartDate','data com que se arrancou o programa')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,9,0,0,'','Warp','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,10,0,1,'','TimeControlInterval','seconds')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,11,0,0,'Europe/Lisbon','TimeControlInterval','seconds')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,12,0,0,'','DbMaintCounter','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,13,0,1580342400,'','DbMaintLastRun','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(0,14,0,20,'','DbMaintDays','')

-- insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,0,0,0,'','schedule_def','0 = NO_SCHEDULE_DEF; 1 = SCHEDULE_DEF ')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,0,0,0,'','in_error','> 0  we have are in error')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,1,0,0,'','in_alert',' > 0 we are in alert mode')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,2,0,0,'NoScheduleDef','state','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,3,0,0,'Standard','mode','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,4,0,20,'','pump_recycle_time','segundos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,5,0,30,'','max_sector_time','minutos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,6,0,40,'','suspend_timeout','minutos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,7,0,4,'','wzrd_water_days_per_week','days/week')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,8,150.0,0,'','wzrd_root_length','mm')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,9,25.0,0,'','wzrd_water_mm_per_week','mm')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,10,0.0,11,'','decrease_alert_level_after','minutos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,11,0.0,6,'','stress_control_interval','minutos')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,12,0.0,0,'','last_stress_control_time','ultima vez que o stress control executou')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,13,0.0,0,'','water_week_acc','mm')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,14,0.0,6,'','water_week_counter','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,15,0.0,1652557995,'','last_save','ultima vez que se gravou')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(2,16,0.0,0,'','live_since','live since')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(3,0,40.440725,0,'','latitude','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(3,1,-8.68294444444444,0,'','longitude','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(3,2,51.0,0,'','elevation','metros')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,0,0,0,'20200813','current_day','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,1,0.0,0,'','rain_probability','%%')
-- insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,2,0.0,0,'','rain_week_acc','mm')
-- insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,3,0,0,'','rain_week_acc_counter','days count')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,4,0,300,'','update_interval','minutos?  testar')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,5,0,0,'a361bd56f8121755c64701f0a08e92cd','secret_key_darksky','')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,6,0.2,0,'','rain_alert_threshold','mm/hora')
insert into 'mods_data' ('module',param,float,int,string,name,descricao ) values(4,7,20.0,0,'','wind_alert_threshold','km/h')


CREATE TABLE  IF NOT EXISTS 'sensor_daily_data' (
'id_metric' INTEGER NOT NULL,
'timestamp' INTEGER NOT NULL,
'value' float NOT NULL,
PRIMARY KEY ('timestamp','id_metric')
) WITHOUT ROWID;

CREATE TABLE  IF NOT EXISTS 'sensor_data' (
'id_sensor' INTEGER NOT NULL,
'timestamp' INTEGER NOT NULL,
'value' float NOT NULL,
PRIMARY KEY ('timestamp','id_sensor')
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS 'scheduled_cycle' (
'id' INTEGER PRIMARY KEY ASC AUTOINCREMENT,
'sim' INTEGER NOT NULL DEFAULT 0,
'cycle_type' INTEGER NOT NULL DEFAULT 4,
'name' TEXT NOT NULL,
'status' INTEGER NOT NULL, 
'current_run' INTEGER  NOT NULL,
'last_run' INTEGER  NOT NULL,
'op' char(1) NOT NULL DEFAULT 'I',
'last_change' INTEGER NOT NULL,
'start_ts' INTEGER  NOT NULL,
'start_sunrise_index' INTEGER NOT NULL,
'start_sunset_index' INTEGER NOT NULL,
'repeat_kind' INTEGER NOT NULL, 
'repeat_spec_wd' INTEGER DEFAULT 0,
'repeat_every_qty' INTEGER NOT NULL,
'repeat_every_unit' INTEGER DEFAULT 0, 
'stop_condition' INTEGER DEFAULT 0,
'stop_retries' INTEGER NOT NULL,
'stop_date_ts' INTEGER  NOT NULL,
'retries_count' INTEGER NOT NULL
);

-- CREATE INDEX 'scheduled_cycle_id_sim' ON 'scheduled_cycle' ('id', 'sim');
-- CREATE INDEX 'scheduled_cycle__start_ts' ON 'scheduled_cycle' ('start_ts');
CREATE INDEX 'scheduled_cycle__status' ON 'scheduled_cycle' ('status');
CREATE INDEX 'scheduled_cycle__start_ts_plus_sim' ON 'scheduled_cycle' ('start_ts', 'sim');
CREATE INDEX 'scheduled_cycle__sim' ON 'scheduled_cycle' ( 'sim');
CREATE INDEX 'scheduled_cycle__cycle_type' ON 'scheduled_cycle' ( 'cycle_type');


CREATE TABLE IF NOT EXISTS 'repeat_kind' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO repeat_kind (id,desc) VALUES ( 0, 'Never' );
INSERT INTO repeat_kind (id,desc) VALUES ( 1, 'Specific Weekday' );
INSERT INTO repeat_kind (id,desc) VALUES ( 2, 'Every' );


CREATE TABLE IF NOT EXISTS 'repeat_spec_wd' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;


CREATE TABLE IF NOT EXISTS 'repeat_every_unit' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO repeat_every_unit (id,desc) VALUES ( 0, 'Seconds' );
INSERT INTO repeat_every_unit (id,desc) VALUES ( 1, 'Minutes' );
INSERT INTO repeat_every_unit (id,desc) VALUES ( 2, 'Hours' );
INSERT INTO repeat_every_unit (id,desc) VALUES ( 3, 'Days' );
INSERT INTO repeat_every_unit (id,desc) VALUES ( 4, 'Weeks' );


CREATE TABLE IF NOT EXISTS 'stop_condition' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO stop_condition (id,desc) VALUES ( 0, 'Never' );
INSERT INTO stop_condition (id,desc) VALUES ( 1, 'Retries' );
INSERT INTO stop_condition (id,desc) VALUES ( 2, 'Date' );

CREATE TABLE IF NOT EXISTS 'sector' (
'id' INTEGER NOT NULL,
'description' TEXT NOT NULL,
'week_acc' float NOT NULL,
'Percolation' float NOT NULL,
'debit' float NOT NULL,
'last_watered_in' INTEGER NOT NULL,
'enabled' INTEGER NOT NULL,
'max_duration' float NOT NULL,
'name' TEXT DEFAULT NULL,
'device_id' INTEGER NOT NULL DEFAULT 65535
'last_change' INTEGER DEFAULT '0',
'op' char(1) DEFAULT 'I', 
PRIMARY KEY ('id')
) WITHOUT ROWID;

DELETE FROM sector;
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (0, 'Zona Sobreiro', 0.0, 0.291041667, 0.001666667, 0, 1, 30, 'Sobreiro', 0, 0, 'I');
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (1, 'Zona Nogueira', 0.0, 0.291041667, 0.001666667, 0, 1, 30, 'Nogueira', 1, 0, 'I');
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (2, 'Zona Deck Sala', 0.0, 0.291041667, 0.001666667, 0, 1, 30, 'Deck Sala', 2, 0, 'I');
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (3, 'Zona Amoreira', 0.0, 0.291041667, 0.001666667, 0, 1, 30, 'Amoreira', 3, 0, 'I');
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (4, 'Zona Traseiras', 0.0, 0.15875, 0.001666667, 0, 1, 30, 'Traseiras', 4, 0, 'I');
INSERT INTO sector (id, Description, Week_ACC, Percolation, Debit, last_watered_in, enabled, max_duration, name, device_id, last_change, op)
VALUES (5, 'Zona Norte', 0.0, 0.291041667, 0.001666667, 0, 1, 30, 'Norte', 5, 0, 'I');


CREATE TABLE IF NOT EXISTS 'sensor' (
'id' INTEGER NOT NULL,
'description' TEXT NOT NULL,
'unit_short' TEXT NOT NULL, 
'last_change' INTEGER DEFAULT NULL,
'op' char(1) DEFAULT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

DELETE FROM sensor;
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (0, 'Rain', 'mm', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (1, 'Temperature', 'ºC', 0, 'I');
-- INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (2, 'Max Temperature', 'ºC', 0, 'I');
-- INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (3, 'Min Temperature', 'ºC', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (2, 'Humidity', '%', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (3, 'Wind Speed', 'km/h', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (4, 'Wind Bearing', 'º', 0, 'I');
-- INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (7, 'EvapoTranspiration', 'mm', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (5, 'Pressure', 'hpa', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (6, 'Solar Radiation', 'Mj/m2', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (7, 'Dew Point', 'ºC', 0, 'I');
INSERT INTO sensor (id, description, unit_short, last_change, op) VALUES (8, 'WaterPumpCurrentDetection', 'bool', 0, 'I');
insert into sensor (id, description, unit_short, last_change, op) values(9,'WattHora', 'W/h', 0, 'I');
-- update ref_ids set next_num = 9 where id = "SENSOR";

CREATE TABLE IF NOT EXISTS 'metric' (
'id' INTEGER NOT NULL,
'description' TEXT NOT NULL,
'unit_short' TEXT NOT NULL, 
'last_change' INTEGER DEFAULT NULL,
'op' char(1) DEFAULT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

DELETE FROM metric;
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (0, 'AvgWindSpeed', 'Km/h', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (1, 'MaxTemp', 'ºC', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (2, 'MinTemp', 'ºC', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (3, 'MaxHumidity', '%', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (4, 'MinHumidity', '%', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (5, 'AvgPressure', 'hpa', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (6, 'MaxPressure', 'hpa', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (7, 'MinPressure', 'hpa', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (8, 'SumRain', 'mm', 0, 'I');
INSERT INTO metric (id, description, unit_short, last_change, op) VALUES (9, 'EvapoTranspiration', 'mm', 0, 'I');


CREATE TABLE IF NOT EXISTS 'watered_cycle' (
'id_ciclo' INTEGER NOT NULL,
'current_run' INTEGER NOT NULL,
'status' INTEGER NOT NULL, -- mudar para int
'start' INTEGER NOT NULL,
'end' INTEGER NOT NULL,
PRIMARY KEY ('id_ciclo','current_run')
) WITHOUT ROWID;

CREATE INDEX 'watered_cycle_status' ON 'watered_cycle' ('status');

CREATE TABLE IF NOT EXISTS 'watered_sector' (
'id_sector' INTEGER NOT NULL,
'id_ciclo' INTEGER NOT NULL,
'current_run' INTEGER NOT NULL,
'minutes_to_water_tgt' float NOT NULL,
'minutes_to_water_acc' float NOT NULL,
'skipped' INTEGER NOT NULL,
'status' INTEGER NOT NULL, -- mudar para int
'start' INTEGER NOT NULL,
'end' INTEGER NOT NULL,
PRIMARY KEY ('id_sector','id_ciclo','current_run')
) WITHOUT ROWID;

CREATE INDEX 'watered_sector_status' ON 'watered_sector' ('status');

CREATE TABLE IF NOT EXISTS aux_mig(
  min_daily_avg INTEGER ,
  min_daily INTEGER,
  min_cycle INTEGER,
  min_sector INTEGER
  ) ;
  
CREATE TABLE 'device' (
'id' INTEGER,
'identifier' TEXT NOT NULL,
'status' INTEGER NOT NULL DEFAULT 0, 
'ip' TEXT NOT NULL, 
'cmd_on' TEXT, 
'cmd_off' TEXT,
'get_status' TEXT,
'desc' TEXT;
'cmd_up' TEXT;
'cmd_stop' TEXT;
'cmd_down' TEXT;
'shutter_get_status' TEXT;
'device_type' INTEGER;
PRIMARY KEY (`id`)
);

DROP INDEX IF EXISTS 'device_identifier';
CREATE INDEX 'device_identifier' ON 'device' ('identifier');

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(0, 'shelly1-485519C96846', 1, '192.168.1.67', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Sobreiro','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(1, 'shelly1-349454764034', 1, '192.168.1.68', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Nogueira','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(2, 'shelly1-349454763185', 1, '192.168.1.69', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Deck sala','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(3, 'shelly1-34945476437A', 1, '192.168.1.70', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Amoreira','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(4, 'shelly1-349454760DF8', 1, '192.168.1.71', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Traseiras','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(5, 'shelly1-C45BBE540D0A', 1, '192.168.1.72', 1, 'relay/0?turn=on', 'relay/0?turn=off', 'status','Norte','','','','',0);


insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(7, 'shelly1-????????????', 0, '192.168.?.?', 'relay/0?turn=on', 'relay/0?turn=off', 'status','Portão pequeno','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(8, 'shelly1-????????????', 0, '192.168.?.?', 'relay/0?turn=on', 'relay/0?turn=off', 'status','Aguas e Aquecimento','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(9, 'shellyswitch25-349454793A10', 1, '192.168.1.17', '', '', 'status','Estore cama - suite','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);  -- roller, que lê consumo e temperatura

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(10, 'shellyswitch25-3494547980B0', 1, '192.168.1.16', '', '', 'status','Estore grande - suite','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(11, 'shellyswitch25-3494547BE08A', 1, '192.168.1.15', '', '', 'status','Estore quarto Ana','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(12, 'shellyswitch25-3494547C12B3', 1, '192.168.1.14', '', '', 'status','Biblioteca','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(13, 'shellyswitch25-3494547BE629', 1, '192.168.1.10', '', '', 'status','Estore Dir. Sala','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(14, 'shellyswitch25-3494547A52B4', 1, '192.168.1.12', '', '', 'status','Estore Esq. Sala','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(15, 'shellyswitch25-3494547A7418', 1, '192.168.1.13', '', '', 'status','Estore Frente Sala','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(16, 'shellyswitch25-????????????', 0, '192.168.?.?', '', '', 'status','Estore pequeno 1º andar','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(17, 'shellyswitch25-3494547A7BAB', 1, '192.168.1.18', '', '', 'status','Estore grande 1º andar','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(18, 'shellyswitch25-3494547BE054', 1, '192.168.1.20', '', '', 'status','Estore Hospedes - pequeno','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(19, 'shellyswitch25-3494547A7A1D', 1, '192.168.1.19', '', '', 'status','Estore Hospedes - grande','roller/0?go=open', 'roller/0?go=stop', 'roller/0?go=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(20, 'shellyht-08b61fcd3b98', 1, '192.168.1.11', '', '', '','Temperatura e Humidade','','','','',70);



DROP TABLE IF EXISTS  'device_type';
CREATE TABLE 'device_type' (
'id' INTEGER,
'desc' TEXT NOT NULL,
PRIMARY KEY (`id`)
);

DROP INDEX IF EXISTS 'device_type_identifier';
CREATE INDEX 'device_type_id' ON 'device' ('id');

DROP TABLE IF EXISTS  'scene';
CREATE TABLE 'scene' (
'id' INTEGER,
'desc' TEXT NOT NULL,
PRIMARY KEY (`id`)
);

insert into scene(id,desc) values(0, 'Abrir portão em proximidade');
insert into scene(id,desc) values(1, 'Todos os estores');
insert into scene(id,desc) values(2, 'Estores da suite');
insert into scene(id,desc) values(3, 'Estores do quarto hospedes');
insert into scene(id,desc) values(4, 'Estores da sala');
insert into scene(id,desc) values(5, 'Estores do 1º Andar');

-- o código é binário para podermos ter combinações de funcionalidades na codificação do device
insert into device_type values(0,'Relay');
insert into device_type values(1,'Shutter');
insert into device_type values(2,'H');
insert into device_type values(4,'T');
insert into device_type values(8,'digital_input');
insert into device_type values(16,'analogic_input');
insert into device_type values(32,'pm');  -- para os devices que também lêm o consumo
insert into device_type values(64,'batery status');  -- para os devices que também reportam estado bateria

DROP TABLE IF EXISTS  'scene_device';
CREATE TABLE 'scene_device' (
'id_scene' INTEGER NOT NULL,
'id_device' INTEGER NOT NULL,
'cmd_run' TEXT NOT NULL, -- para imaginar que posso ter coisas como "run" ou "stop"
'cmd_stop' TEXT NOT NULL, -- para imaginar que posso ter um cenário em execução e posso querer pará-lo
'cmd_open' TEXT NOT NULL, -- para os estores
'cmd_close' TEXT NOT NULL, -- para os estores
-- 'is_timed' INTEGER,
PRIMARY KEY ('id_scene', 'id_device')
);


insert into scene_device(id_scene, id_device, cmd_run, cmd_stop, cmd_open, cmd_close) values(0,6,'run','','','');

insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,9,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,10,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,11,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,12,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,13,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,14,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,15,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,16,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,17,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,18,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(1,19,'','open','stop','close');

insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(2,9,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(2,10,'','open','stop','close');

insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(3,18,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(3,19,'','open','stop','close');

insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(4,13,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(4,14,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(4,15,'','open','stop','close');

insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(5,16,'','open','stop','close');
insert into scene_device(id_scene, id_device, cmd_run,cmd_stop, cmd_open, cmd_close) values(5,17,'','open','stop','close');

//TODO - avaliar se é necessário atualizar o dw nos temas dos sensores
----------------------------------------   DW CREATION   ------------------------------------------------
ATTACH DATABASE 'dw.db' AS dw;

CREATE TABLE IF NOT EXISTS water_state (
    'id'     INTEGER      NOT NULL,
    'status' TEXT         NOT NULL
    PRIMARY KEY('id')
) WITHOUT ROWID;

DELETE FROM water_state;
INSERT INTO dw.water_state SELECT * FROM db.water_state;


CREATE TABLE  IF NOT EXISTS 'sensor_daily_data' (
'id_metric' INTEGER NOT NULL,
'timestamp' INTEGER NOT NULL,
'value' float NOT NULL,
PRIMARY KEY ('timestamp','id_metric')
) WITHOUT ROWID;

CREATE TABLE  IF NOT EXISTS 'sensor_data' (
'id_sensor' INTEGER NOT NULL,
'timestamp' INTEGER NOT NULL,
'value' float NOT NULL,
PRIMARY KEY ('timestamp','id_sensor')
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS 'scheduled_cycle' (
'id' INTEGER PRIMARY KEY ASC AUTOINCREMENT,
'sim' INTEGER NOT NULL DEFAULT 0,
'cycle_type' INTEGER NOT NULL DEFAULT 4,
'name' TEXT NOT NULL,
'status' INTEGER NOT NULL, 
'current_run' INTEGER  NOT NULL,
'last_run' INTEGER  NOT NULL,
'op' char(1) NOT NULL DEFAULT 'I',
'last_change' INTEGER NOT NULL,
'start_ts' INTEGER  NOT NULL,
'start_sunrise_index' INTEGER NOT NULL,
'start_sunset_index' INTEGER NOT NULL,
'repeat_kind' INTEGER NOT NULL, 
'repeat_spec_wd' INTEGER DEFAULT 0,
'repeat_every_qty' INTEGER NOT NULL,
'repeat_every_unit' INTEGER DEFAULT 0, 
'stop_condition' INTEGER DEFAULT 0,
'stop_retries' INTEGER NOT NULL,
'stop_date_ts' INTEGER  NOT NULL,
'retries_count' INTEGER NOT NULL
);

-- CREATE INDEX 'scheduled_cycle_id_sim' ON 'scheduled_cycle' ('id', 'sim');
-- CREATE INDEX 'scheduled_cycle__start_ts' ON 'scheduled_cycle' ('start_ts');
CREATE INDEX 'scheduled_cycle__status' ON 'scheduled_cycle' ('status');
CREATE INDEX 'scheduled_cycle__start_ts_plus_sim' ON 'scheduled_cycle' ('start_ts', 'sim');
CREATE INDEX 'scheduled_cycle__sim' ON 'scheduled_cycle' ( 'sim');
CREATE INDEX 'scheduled_cycle__cycle_type' ON 'scheduled_cycle' ( 'cycle_type');

CREATE TABLE IF NOT EXISTS 'repeat_kind' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO dw.repeat_kind SELECT * from db.repeat_kind;

CREATE TABLE IF NOT EXISTS 'repeat_spec_wd' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO dw.repeat_spec_wd SELECT * from db.repeat_spec_wd;

CREATE TABLE IF NOT EXISTS 'repeat_every_unit' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO dw.repeat_every_unit SELECT * from db.repeat_every_unit;

CREATE TABLE IF NOT EXISTS 'stop_condition' (
'id' INTEGER NOT NULL,
'desc' TEXT NOT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;

INSERT INTO dw.stop_condition SELECT * from db.stop_condition;


CREATE TABLE IF NOT EXISTS 'sector' (
'id' INTEGER NOT NULL,
'description' TEXT NOT NULL,
'week_acc' float NOT NULL,
'Percolation' float NOT NULL,
'debit' float NOT NULL,
'last_watered_in' INTEGER NOT NULL,
'enabled' INTEGER NOT NULL,
'max_duration' float NOT NULL,
'name' TEXT DEFAULT NULL,
'last_change' INTEGER DEFAULT '0',
'op' char(1) DEFAULT 'I',
PRIMARY KEY ('id')
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS 'sensor' (
'id' INTEGER NOT NULL,
'description' TEXT NOT NULL,
'unit_short' TEXT NOT NULL,
'last_change' INTEGER DEFAULT NULL,
'op' char(1) DEFAULT NULL,
PRIMARY KEY ('id')
) WITHOUT ROWID;


CREATE TABLE IF NOT EXISTS 'watered_cycle' (
'id_ciclo' INTEGER NOT NULL,
'current_run' INTEGER NOT NULL,
'status' INTEGER NOT NULL,-- mudar para int
'start' INTEGER NOT NULL,
'end' INTEGER NOT NULL,
PRIMARY KEY ('id_ciclo','current_run')
) WITHOUT ROWID;

CREATE INDEX 'watered_cycle_status' ON 'watered_cycle' ('status');

CREATE TABLE IF NOT EXISTS 'watered_sector' (
'id_sector' INTEGER NOT NULL,
'id_ciclo' INTEGER NOT NULL,
'current_run' INTEGER NOT NULL,
'minutes_to_water_tgt' float NOT NULL,
'minutes_to_water_acc' float NOT NULL,
'skipped' INTEGER NOT NULL,
'status' INTEGER NOT NULL,-- mudar para int
'start' INTEGER NOT NULL,
'end' INTEGER NOT NULL,
PRIMARY KEY ('id_sector','id_ciclo','current_run')
) WITHOUT ROWID;

CREATE INDEX 'watered_sector_status' ON 'watered_sector' ('status');


COMMIT;
PRAGMA ignore_check_constraints = ON;
PRAGMA foreign_keys = ON;
-- PRAGMA journal_mode = WAL;
-- PRAGMA synchronous = NORMAL;


--- SCRIPT PARA A MIGRAÇÃO/MANUTENÇÃO A CADA X DIAS DA BD - ESTÁ AQUI COMO EXEMPLO
BEGIN

DELETE from db.aux_mig;
INSERT INTO db.aux_mig (min_daily_avg, min_daily, min_cycle, min_sector) 
VALUES (SELECT max(timestamp) from dw.daily_measure_avg,
        SELECT Max(timestamp) from dw.daily_measure,
        SELECT Max(current_run) from dw.watered_cycle,
        SELECT Max(current_run) from dw.watered_sector);

delete from dw.sector;
delete from dw.sensor;
insert into dw.sector select * from db.sector;
insert into dw.sensor select * from db.sensor;

insert into dw.daily_measure_avg select * from db.daily_measure_avg
where timestamp<=SELECT min_daily_avg from dw.aux_mig and timestamp>?;
insert into dw.daily_measure select * from db.daily_measure
where timestamp<=SELECT min_daily from dw.aux_mig and timestamp>?;
insert into dw.watered_cycle select * from db.watered_cycle 
where current_run<=SELECT min_cycle from dw.aux_mig and current_run>?;
insert into dw.watered_sector select * from db.watered_sector 
where current_run<=SELECT min_sector from dw.aux_mig and current_run>?;

delete from db.daily_measure_avg where timestamp<=SELECT min_daily_avg from dw.aux_mig;
delete from db.daily_measure where timestamp<=SELECT min_daily from dw.aux_mig;
delete from db.watered_cycle where current_run<=SELECT min_cycle from dw.aux_mig;
delete from db.watered_sector where current_run<=SELECT min_sector from dw.aux_mig;

COMMIT;
