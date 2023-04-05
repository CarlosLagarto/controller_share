select * from device;

insert into device(id,identifier,status,ip,cmd_on,cmd_off,get_status, desc, cmd_up, cmd_stop, cmd_down, shutter_get_status, device_type) 
values(10, 'shellyswitch25-3494547980B0', 1, '192.168.1.16', '', '', 'status','Estore grande - suite','roller/0?command=open','roller/0?command=stop','roller/0?command=close','status',37);

update device set cmd_up="roller/0?go=open", cmd_stop="roller/0?go=stop", cmd_down="roller/0?go=close" where device_type=37;



select * from device_type;

select * from device;

select * from scene;
select * from scene_device;

select id_device from scene_device where id_scene=0;

select id,desc from scene order by id;
select id,identifier,status,ip,cmd_on, cmd_off, get_status from device where id<=5;

select id,identifier,status,ip,cmd_on,cmd_off,get_status,desc,cmd_up,cmd_stop,cmd_down,shutter_get_status,device_type from device where id>5;
