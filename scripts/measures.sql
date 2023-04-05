use watering;
SELECT idsensor, floor(timestamp / 60) as minutets, avg(value) as avg_value FROM dailymeasures  inner JOIN sensor ON idsensor = sensor.id  
where timestamp > 1574899141 and timestamp <= 1574908799 and (idsensor = 1 or idsensor = 8) group by idsensor, minutets order by idsensor, minutets;

use watering;
select  t1.minutets, 
		floor(1574908799/60) - t1.minutets as diff,
		coalesce(avg(case when idsensor = 1 then avg_value end), 0) as temp,
		coalesce(avg(case when idsensor = 8 then avg_value end), 0) as press
from (SELECT idsensor, floor(timestamp / 60) as minutets, avg(value) as avg_value FROM dailymeasures  inner JOIN sensor ON idsensor = sensor.id  
where timestamp > 1574899141 and timestamp <= 1574908799 and (idsensor = 1 or idsensor = 8) group by idsensor, minutets order by idsensor, minutets) as t1
group by t1.minutets order by t1.minutets;

use watering;
select  t1.minutets, 
		26254108 - t1.minutets as diff, 
        coalesce(avg(case when idsensor = 5 then avg_value end), 0) as temp, 
        coalesce(avg(case when idsensor = 6 then avg_value end), 0) as press 
from (SELECT idsensor, floor(timestamp / 60) as minutets, avg(value) as avg_value FROM dailymeasures inner JOIN sensor ON idsensor = sensor.id 
where timestamp > 1574899141 and timestamp <= 1574908799 and (idsensor = 5 or idsensor = 6)  group  by idsensor, minutets order by idsensor, minutets) as t1 
group by t1.minutets order by t1.minutets;

use watering;
select  t1.minutets, 
		floor(1574908799/60) - t1.minutets as diff,
		coalesce(avg(case when idsensor = 5 then avg_value end), 0) as speed,
		coalesce(avg(case when idsensor = 6 then avg_value end), 0) as bearing
from (SELECT idsensor, floor(timestamp / 60) as minutets, avg(value) as avg_value FROM dailymeasures  inner JOIN sensor ON idsensor = sensor.id  
where timestamp > 1574899141 and timestamp <= 1574908799 and (idsensor = 5 or idsensor = 6) group by idsensor, minutets order by idsensor, minutets) as t1
group by t1.minutets order by t1.minutets;