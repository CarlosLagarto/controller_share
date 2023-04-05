"use strict";

//TESTED
function time_controller(pump_recycle_time, callback) {

	this.callback = callback;
	this.pump_recycle_time = pump_recycle_time;

	this.time = null;
	const WAIT_TIME = 2000;

	this.cycleId = -1;
	this.sectorId = -1;

	const duration = (m, d) => m * 60 + d;
	const perc = (d, e, s) => d / (e - s);

	this.timerCycle = function(){
		let cycle = DB.cycles[this.cycleId];
        let elapsedSeconds = get_unix_now_time() - cycle.start;
		cycle.exec_perc = perc(elapsedSeconds, cycle.end, cycle.start);
		if (cycle.exec_perc > 1){
			cycle.exec_perc = 1.0;
		}
		let sector = null;
		let sec_start = 0;
		for(let sID in DB.sectors){
            sector = DB.sectors[sID];
			if (sector.is_running()){
	            sec_start = sector.start;
                elapsedSeconds = get_unix_now_time() - sec_start;
	            sector.watering_percent = perc(elapsedSeconds, sector.end, sec_start);
	            if (sector.watering_percent > 1){
	            	sector.watering_percent = 1.0;
	            }
				break;
			}
		}
		if (cycle.exec_perc < 1.0 && cycle.is_running()){
			this.time = setTimeout(this.timerCycle, WAIT_TIME);
		}else{
			this.stopCycle();
		}
		this.callback();
		sector = null;
		cycle = null;
	}.bind(this);

	this.timerSector = function(){
		let sector = DB.sectors[this.sectorId];
        let sectorElapsedSeconds = get_unix_now_time() - sector.start;
        sectyor.watering_percent = perc(sectorElapsedSeconds, sector.end, sector.start);

		if (sector.watering_percent < 1.0 && s.is_running()){
			this.time = setTimeout(this.timerSector, WAIT_TIME);
		}else{
			this.stopSector();
		}
		this.callback();
		sector = null;	
	}.bind(this);

	this.startCycle = function(cycleId){
        let self = this;
		if (!self.time){
			self.cycleId = cycleId;
			let cycle = DB.cycles[cycleId];
			let s = null;
			let totTime = 0;
			let dur = 0.0;
	        for (let row_id in DB.sectors){
	            s = DB.sectors[row_id];
	            if (s.enabled){
	            	dur = duration(s.minutes_to_water, self.pump_recycle_time);
		            totTime = totTime + dur;
		        }
	        }
	        totTime = totTime - self.pump_recycle_time; // we have one extra add above, so, to not have one more if inside the loop, just subtract here.
	        cycle.end = cycle.start + totTime;
			self.timerCycle();
			s = null;
			cycle = null;
		}
        self = null;
	}.bind(this);

	this.startSector = function(sectorId){
        let self = this;
		if(!self.time){
			self.sectorId = sectorId;
	        let s = DB.sectors[self.sectorId];
	        self.start = get_unix_now_time();
			self.timerSector();
			s = null;
		}
        self = null;
	}.bind(this);

	this._cleanSectors = function(){
		let s = null;
		for(let sID in DB.sectors){
            s = DB.sectors[sID];
            s.watering_percent = 0.0;
		}		
	}.bind(this);

	this.stopCycle = function(){
        let self = this;
		clearTimeout(self.time);
		self._cleanSectors();
		self.cycleId = -1;
		self.time = null;
        self = null;
	}.bind(this);

	this.stopSector = function(){
        let self = this;
		clearTimeout(self.time);
		self._cleanSectors();
		self.sectorId = -1;
		self.time = null;
        self = null;
	}.bind(this);

}
