"use strict"

//TESTED
function time_controller(pump_recycle_time, callback) {

	this.callback = callback;
	this.pump_recycle_time = pump_recycle_time;

	this.time = null;
	const WAIT_TIME = 2000;

	this.cycleId = -1;
	this.sectorId = -1;

	const duration = (m, d) => m * 60 + d;
	//const exec_perc = (t, m, d) => exec(t, m, d) * 100;
	const perc = (d, e, s) => d / (e - s);

	this.timerCycle = function(){
		let cycle = DB.cycles[this.cycleId];
		let elapsedSeconds = moment().unix() - cycle.start_ts;
		cycle.exec_perc = perc(elapsedSeconds, cycle.end_ts, cycle.start_ts);
		if (cycle.exec_perc > 1){
			cycle.exec_perc = 1.0;
		}
		let s = null;
		let sec_start = 0;
		for(let sID in DB.sectors){
            s = DB.sectors[sID];
			if (s.status === WATERING_STATUS.RUNNING){
	            sec_start = s.start_utc_ts;
	            elapsedSeconds = moment().unix() - sec_start;
	            s.watering_percent = perc(elapsedSeconds, s.end_utc_ts, sec_start);
	            if (s.watering_percent > 1){
	            	s.watering_percent = 1.0;
	            }
				break;
			}
		}
		if (cycle.exec_perc < 1.0 && cycle.status === WATERING_STATUS.RUNNING){
			this.time = setTimeout(this.timerCycle, WAIT_TIME);
		}else{
			this.stopCycle();
		}
		this.callback();
		s = null;
		cycle = null;
	}.bind(this);

	this.timerSector = function(){
		let s = DB.sectors[this.sectorId];
        let sectorElapsedSeconds = moment().unix() - s.start_utc_ts;
        s.watering_percent = perc(sectorElapsedSeconds, s.end_utc_ts, s.start_utc_ts);

		if (s.watering_percent < 1.0 && s.status === WATERING_STATUS.RUNNING){
			this.time = setTimeout(this.timerSector, WAIT_TIME);
		}else{
			this.stopSector();
		}
		this.callback();
		s = null;	
	}.bind(this);

	this.startCycle = function(cycleId){
		if (!this.time){
			this.cycleId = cycleId;
			let cycle = DB.cycles[cycleId];
			let s = null;
			let totTime = 0;
			let dur = 0.0;
	        for (let row_id in DB.sectors){
	            s = DB.sectors[row_id];
	            if (s.enabled){
	            	dur = duration(s.minutes_to_water, this.pump_recycle_time);
		            totTime = totTime + dur;
		        }
	        }
	        totTime = totTime - this.pump_recycle_time; // lá em cima somamos mais uma vez do que o preciso.  Para não por um if lá em cima, subtraio aqui.
	        cycle.end_ts = cycle.start_ts + totTime;
			this.timerCycle();
			s = null;
			cycle = null;
		}
	}.bind(this);

	this.startSector = function(sectorId){
		if(!this.time){
			this.sectorId = sectorId;
			//let totTime = 0;
	        let s = DB.sectors[this.sectorId];
	        this.start = moment().unix();
			this.timerSector();
			s = null;
		}
	}.bind(this);

	this._cleanSectors = function(){
		let s = null;
		for(let sID in DB.sectors){
            s = DB.sectors[sID];
            s.watering_percent = 0.0;
		}		
	}.bind(this);

	this.stopCycle = function(){
		clearTimeout(this.time);
		this._cleanSectors();
		this.cycleId = -1;
		this.time = null;
	}.bind(this);

	this.stopSector = function(){
		clearTimeout(this.time);
		this._cleanSectors();
		this.sectorId = -1;
		this.time = null;
	}.bind(this);

}
