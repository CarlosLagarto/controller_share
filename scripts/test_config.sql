
select * from mods_data where module = 4 order by param;
update mods_data set string = null, float = null where module = 4 and param = 12;

SELECT float,int,string FROM mods_data where module=2 order by param;
SELECT * FROM mods_data where module=2 order by param;


update mods_data set float=3.5714285, int=null where module = 2 and param=10;