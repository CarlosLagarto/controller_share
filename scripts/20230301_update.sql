
-- SCRIPT para atualizar produção - alterações - em curso depois de 2023-01-22
-- ISTO JÁ ESTÁ EM DEV, E EM PRE-PROD

alter table device add column desc TEXT;
alter table device add column cmd_up TEXT;
alter table device add column cmd_stop TEXT;
alter table device add column cmd_down TEXT;
alter table device add column shutter_get_status TEXT;
alter table device add column device_type INTEGER;


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

-- o código é binário para podermos ter combinações de funcionalidades na codificação do device
insert into device_type values(0,'Relay');
insert into device_type values(1,'Shutter');
insert into device_type values(2,'H');
insert into device_type values(4,'T');
insert into device_type values(8,'digital_input');
insert into device_type values(16,'analogic_input');
insert into device_type values(32,'pm');  -- para os devices que também lêm o consumo
insert into device_type values(64,'batery status');  -- para os devices que também reportam estado bateria


update device set device_type = 0 where id = 0;
update device set device_type = 0 where id = 1;
update device set device_type = 0 where id = 2;
update device set device_type = 0 where id = 3;
update device set device_type = 0 where id = 4;
update device set device_type = 0 where id = 5;


update device set desc='Sobreiro',device_type=0 where id = 0;
update device set desc='Nogueira',device_type=0 where id = 1;
update device set desc='Deck Sala',device_type=0 where id = 2;
update device set desc='Amoreira',device_type=0 where id = 3;
update device set desc='Traseiras',device_type=0 where id = 4;
update device set desc='Norte',device_type=0 where id = 5;


insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(6, 'shelly1-????????????', 0, '192.168.?.?', 'relay/0?turn=on', 'relay/0?turn=off', 'status','Portão grande','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(7, 'shelly1-????????????', 0, '192.168.?.?', 'relay/0?turn=on', 'relay/0?turn=off', 'status','Portão pequeno','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(8, 'shelly1-????????????', 0, '192.168.?.?', 'relay/0?turn=on', 'relay/0?turn=off', 'status','Aguas e Aquecimento','','','','',0);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(9, 'shellyswitch25-349454793A10', 1, '192.168.1.17', '', '', 'status','Estore cama - suite','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);  -- roller, que lê consumo e temperatura

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(10, 'shellyswitch25-3494547980B0', 1, '192.168.1.16', '', '', 'status','Estore grande - suite','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(11, 'shellyswitch25-3494547BE08A', 1, '192.168.1.15', '', '', 'status','Estore quarto Ana','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(12, 'shellyswitch25-3494547C12B3', 1, '192.168.1.14', '', '', 'status','Biblioteca','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(13, 'shellyswitch25-3494547BE629', 1, '192.168.1.10', '', '', 'status','Estore Dir. Sala','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(14, 'shellyswitch25-3494547A52B4', 1, '192.168.1.12', '', '', 'status','Estore Esq. Sala','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(15, 'shellyswitch25-3494547A7418', 1, '192.168.1.13', '', '', 'status','Estore Frente Sala','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(16, 'shellyswitch25-????????????', 0, '192.168.?.?', '', '', 'status','Estore pequeno 1º andar','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(17, 'shellyswitch25-3494547A7BAB', 1, '192.168.1.18', '', '', 'status','Estore grande 1º andar','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(18, 'shellyswitch25-3494547BE054', 1, '192.168.1.20', '', '', 'status','Estore Hospedes - pequeno','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(19, 'shellyswitch25-3494547A7A1D', 1, '192.168.1.19', '', '', 'status','Estore Hospedes - grande','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(20, 'shellyht-08b61fcd3b98', 1, '192.168.1.11', '', '', '','Temperatura e Humidade','','','','',70);

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

insert into scene(id,desc) values(0, 'Abrir portão em proximidade');
insert into scene(id,desc) values(1, 'Todos os estores');
insert into scene(id,desc) values(2, 'Estores da suite');
insert into scene(id,desc) values(3, 'Estores do quarto hospedes');
insert into scene(id,desc) values(4, 'Estores da sala');
insert into scene(id,desc) values(5, 'Estores do 1º Andar');

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
-- A ideia dos schedules é o cenário de executar cenários num determinado momento ou sofisticar a coisa e coordenar 
-- os devices de um cenário de alguma forma tendo em atenção o tempo.  
-- Como isso só vai ser perceptivel quando tiver a necessidade, para já trabalha-se só com as cenas, e no código com as customizações,
-- até ter mais sensibilidade para as necessidades, para não andar a fazer código só porque sim, por ideias que podem nunca acontecer.
-- 'start_ts' INTEGER,
-- 'start_sunrise_index' INTEGER,
-- 'start_sunset_index' INTEGER ,
-- 'repeat_kind' INTEGER, 
-- 'repeat_spec_wd' INTEGER DEFAULT 0,
-- 'repeat_every_qty' INTEGER ,
-- 'repeat_every_unit' INTEGER DEFAULT 0, 
-- 'stop_condition' INTEGER DEFAULT 0,
-- 'stop_retries' INTEGER ,
-- 'stop_date_ts' INTEGER ,
-- 'retries_count' INTEGER,
