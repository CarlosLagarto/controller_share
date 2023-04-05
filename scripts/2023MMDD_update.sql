---  PARA A NOVA ATUALIZAÇÃO
select * from mods_data where module = 0 order by param;
delete from mods_data where module = 0 and param = 9;
update mods_data set param = 9 where module = 0 and param = 10;
update mods_data set param = 10 where module = 0 and param = 11;
update mods_data set param = 11 where module = 0 and param = 12;
update mods_data set param = 12 where module = 0 and param = 13;
update mods_data set param = 13 where module = 0 and param = 14;

update mods_data set float=3.5714285, int=null where module = 2 and param=10;