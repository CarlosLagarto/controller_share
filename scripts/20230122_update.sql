-- SCRIPT para atualizar produção - alterações - aplicado em pre-prod em 2023-01-22
------------- ALTERAÇÕES A DB
--- STEP 1
delete from mods_data where module = 0 and param = 9;
update mods_data set param = 9 where module = 0 and param = 10;
update mods_data set param = 10 where module = 0 and param = 11;
update mods_data set param = 11 where module = 0 and param = 12;
update mods_data set param = 12 where module = 0 and param = 13;
update mods_data set param = 13 where module = 0 and param = 14;
update mods_data set param = 14 where module = 0 and param = 15;

--- STEP 2
insert into sensor values(9,'WattHora', 'W/h', 0, 'I');

--- STEP 3
DROP TABLE IF EXISTS  'device';
CREATE TABLE 'device' (
'id' INTEGER,
'identifier' TEXT NOT NULL,
'status' INTEGER NOT NULL DEFAULT 0, 
'ip' TEXT NOT NULL, 
'cmd_on' TEXT, 
'cmd_off' TEXT,
'get_status' TEXT,
PRIMARY KEY (`id`)
);

DROP INDEX IF EXISTS 'device_identifier';
CREATE INDEX 'device_identifier' ON 'device' ('identifier');

insert into device values(0, 'shelly1-485519C96846', 1, '192.168.1.67', 'relay/0?turn=on', 'relay/0?turn=off', 'status');

--- STEP 4
alter table sector add column device_id INTEGER NOT NULL DEFAULT 65535;
update sector set device_id = 0 where id = 0;

--- STEP 5 - estou a pôr todos os setores a apontar para o mesmo device apenas para testes 
---        - em produção isto tem que se ajustar para a realidade dos ids dos devices
update sector set device_id = 0 where id = 1;
update sector set device_id = 0 where id = 2;
update sector set device_id = 0 where id = 3;
update sector set device_id = 0 where id = 4;
update sector set device_id = 0 where id = 5;

-------------------------

---- STEP 6

insert into device values(1, 'shelly1-349454764034', 1, '192.168.1.68', 'relay/0?turn=on', 'relay/0?turn=off', 'status');
insert into device values(2, 'shelly1-349454763185', 1, '192.168.1.69', 'relay/0?turn=on', 'relay/0?turn=off', 'status');
insert into device values(3, 'shelly1-34945476437A', 1, '192.168.1.70', 'relay/0?turn=on', 'relay/0?turn=off', 'status');
insert into device values(4, 'shelly1-349454760DF8', 1, '192.168.1.71', 'relay/0?turn=on', 'relay/0?turn=off', 'status');
insert into device values(5, 'shelly1-C45BBE540D0A', 1, '192.168.1.72', 'relay/0?turn=on', 'relay/0?turn=off', 'status');

update sector set device_id = 1 where id = 1;
update sector set device_id = 2 where id = 2;
update sector set device_id = 3 where id = 3;
update sector set device_id = 4 where id = 4;
update sector set device_id = 5 where id = 5;

------------- ALTERAÇÕES A DW
--- STEP 1
insert into sensor values(9,'WattHora', 'W/h', 0, 'I');

--- STEP 2
alter table sector add column device_id INTEGER NOT NULL DEFAULT 65535;
update sector set device_id = 0 where id = 0;

--- STEP 3 - estou a pôr todos os setores a apontar para o mesmo device apenas para testes 
---        - em produção isto tem que se ajustar para a realidade dos ids dos devices
update sector set device_id = 0 where id = 1;
update sector set device_id = 0 where id = 2;
update sector set device_id = 0 where id = 3;
update sector set device_id = 0 where id = 4;
update sector set device_id = 0 where id = 5;

-------------------------

---- STEP 4

update sector set device_id = 1 where id = 1;
update sector set device_id = 2 where id = 2;
update sector set device_id = 3 where id = 3;
update sector set device_id = 4 where id = 4;
update sector set device_id = 5 where id = 5;
