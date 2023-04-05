"use strict";

//TESTED
function Row(){

    this.set = function(row){
        let self = this;
        for (let attr in row) {
            if (typeof(row[attr]) !== "undefined" && typeof(row[attr]) !== "function" && typeof(row[attr]) !== "object"){
                self[attr] = row[attr];
            }
        }
    };


    this.conditionalSet = function(row){  
        let changedFields = [];
        let self = this;
        for (let attr in row) {
            if (typeof(row[attr]) !== "undefined" && typeof(row[attr]) !== "function" && typeof(row[attr]) !== "object"){
                if (self[attr] !== row[attr]){
                    self[attr] = row[attr];
                    changedFields.push(attr);
                }
            }
        }

        return changedFields;
    };

    this.insFO = function(obj){ 
        this.set(obj);
    };

    this.upd = function(row){  
        return this.conditionalSet(row);  
    };
}

//TESTED
function Config(){
    Row.call(this);
    this.prototype = Object.create(Row.prototype)
    Object.defineProperty(this.prototype, 'constructor', {value: this, enumerable: false, writable: true });

    this.last_sync = 0;
    this.last_change = 0;

    this.mode = "";
    this.in_error = 0;
    this.in_alert = 0;
    this.date = "";

    this.max_sector_time = 0;
    this.pump_recycle_time = 0;
    this.schedule_def = 0;
    this.strategy_water_mm_per_week = 0;
    this.strategy_water_days_per_week = 0;
    this.strategy_root_length= 0;
    this.rain_alert_threshold = 0;
    this.wind_alert_threshold = 0;
    this.watering_suspend_timeout = 0;
    this.decrease_alert_level_after = 0;
    this.stress_control_interval = 0;

    this.db_maint_days = 0;
    this.db_maint_counter = 0;
    this.weekly_acc_precipitation = 0;
    this.rain_week_acc_days = 0;
    this.rain_week_acc_counter = 0;

    this.live_since = "";
    this.current_state = "";
    this.simulation = 0;

    this.ins = function( mode,
                         in_error,
                         in_alert,
                         date,
                         max_sector_time,
                         pump_recycle_time,
                         schedule_def,
                         strategy_water_mm_per_week,
                         strategy_water_days_per_week,
                         strategy_root_length,
                         rain_alert_threshold,
                         wind_alert_threshold,
                         watering_suspend_timeout,
                         decrease_alert_level_after,
                         stress_control_interval,
                         db_maint_days,
                         db_maint_counter,
                         weekly_acc_precipitation,
                         rain_week_acc_days,
                         rain_week_acc_counter,
                         live_since,
                         current_state,
                         simulation,
                         last_sync,
                         last_change){
        this.insFO({mode                            : mode,
                    in_error                        : in_error,
                    in_alert                        : in_alert,
                    date                            : date,
                    max_sector_time                 : max_sector_time,
                    pump_recycle_time               : pump_recycle_time,
                    schedule_def                    : schedule_def,
                    strategy_water_mm_per_week     : strategy_water_mm_per_week,
                    strategy_water_days_per_week: strategy_water_days_per_week,
                    strategy_root_length        : strategy_root_length,
                    rain_alert_threshold            : rain_alert_threshold,
                    wind_alert_threshold            : wind_alert_threshold,
                    watering_suspend_timeout        : watering_suspend_timeout,
                    decrease_alert_level_after      : decrease_alert_level_after,
                    stress_control_interval         : stress_control_interval,
                    db_maint_days                   : db_maint_days,
                    db_maint_counter                : db_maint_counter,
                    weekly_acc_precipitation          : weekly_acc_precipitation,
                    rain_week_acc_days              : rain_week_acc_days,
                    rain_week_acc_counter           : rain_week_acc_counter,
                    live_since                      : live_since,
                    current_state                   : current_state,
                    simulation                      : simulation,
                    last_sync                       : last_sync,
                    last_change                     : last_change})
    };

    this.clone = function(){
        let o = new Config();
        o.set({ mode                                : this.mode,
                in_error                            : this.in_error,
                in_alert                            : this.in_alert,
                date                                : this.date,
                max_sector_time                     : this.max_sector_time,
                pump_recycle_time                   : this.pump_recycle_time,
                schedule_def                        : this.schedule_def,
                strategy_water_mm_per_week         : this.strategy_water_mm_per_week,
                strategy_water_days_per_week    : this.strategy_water_days_per_week,
                strategy_root_length            : this.strategy_root_length,
                rain_alert_threshold                : this.rain_alert_threshold,
                wind_alert_threshold                : this.wind_alert_threshold,
                watering_suspend_timeout            : this.watering_suspend_timeout,
                decrease_alert_level_after          : this.decrease_alert_level_after,
                stress_control_interval             : this.stress_control_interval,
                db_maint_days                       : this.db_maint_days,
                db_maint_counter                    : this.db_maint_counter,
                weekly_acc_precipitation              : this.weekly_acc_precipitation,
                rain_week_acc_days                  : this.rain_week_acc_days,
                rain_week_acc_counter               : this.rain_week_acc_counter,
                live_since                          : this.live_since,
                current_state                       : this.current_state,
                simulation                          : this.simulation,
                last_sync                           : this.last_sync,
                op                                  : this.op,
                last_change                         : this.last_change});
        return o;
    };
}

//TESTED
function Cycle(){
    Row.call(this);
    this.prototype = Object.create(Row.prototype)

    Object.defineProperty(this.prototype, 'constructor', {value: this, enumerable: false, writable: true });    
    this.last_change = 0;
    this.op = "I";

    this.nome = "";
    this.id = 0;
    this.start_ts = 0;
    this.last_run_ts = 0;
    this.status = "";

    this.repeat_kind = "";  //# never/hourly/daily/weekday/specific week days/weekly/monthly/every
    this.repeat_spec_wd = ""; //  # "", #Su|Mo|Tu|We|Th|Fr|St
    this.repeat_every_qty = 0;
    this.repeat_every_unit = ""; //  # "", #minutes, hours, days, week, month
    this.stop_condition = ""; //  # "never", #never, x-retries, date
    this.stop_retries = 0;
    this.stop_date_ts = 0; //  # 0 #timestamp
    this.retries_count = 0;
    this.start_sunrise_index = 1;
    this.start_sunset_index = 0;

    this.end_ts = 0;
    this.exec_perc = 0.0;

    this.ins = function(nome, 
                        id,
                        start_ts, 
                        last_run_ts,
                        repeat_kind,
                        repeat_spec_wd,
                        repeat_every_qty,
                        repeat_every_unit,
                        repeat_stop_after, 
                        stop_retries,
                        stop_date_ts,
                        retries_count,
                        start_sunrise_index,
                        start_sunset_index,
                        status,
                        last_change,
                        op){
        this.insFO({nome                : nome,
                    id                  : id,
                    start_ts            : start_ts,
                    last_run_ts         : last_run_ts,
                    repeat_kind         : repeat_kind,
                    repeat_spec_wd      : repeat_spec_wd,
                    repeat_every_qty    : repeat_every_qty,
                    repeat_every_unit   : repeat_every_unit,
                    repeat_stop_after   : repeat_stop_after, 
                    stop_retries        : stop_retries,
                    stop_date_ts        : stop_date_ts,
                    retries_count       : retries_count, 
                    start_sunrise_index : start_sunrise_index,
                    start_sunset_index  : start_sunset_index,
                    status              : status, 
                    last_change         : last_change,
                    op                  : op})
    };

    this.start_ts_str = function(){
        return moment.unix(this.start_ts).format(DATE_LONG_FORMAT);
    };
    this.stop_date_ts_str = function(){
        return moment.unix(this.stop_date_ts).format(DATE_LONG_FORMAT);
    };
    this.prev_cycle_description = function(){
        return moment.unix(this.last_run_ts).fromNow();
    };
    this.next_cycle_description = function(){
        return moment.unix(this.start_ts).fromNow();
    };

    this.start_str = function(){
        return moment.unix(this.start_ts).format(TIME_24H_FORMAT);
    };
    this.end_str = function(){
        return moment.unix(this.end_ts).format(TIME_24H_FORMAT);
    };

    this.start_exec_perc_str = function(){
        return round((100 * this.exec_perc),0).toString();
    };

    this.week_days = function(){
        if (this.repeat_spec_wd.length > 0){
            return this.repeat_spec_wd.split("|");
        }else{
            return [];
        }
    };

    //NOTTESTED
    this.description = function(){
        //var msg = "";
        let _msg = [];
        let self = this;
        switch(this.repeat_kind){
            case SCHEDULE_REPEAT.NEVER:
                _msg.push(`Regou uma vez em ${moment.unix(self.start_ts).format()}`);  //A. regou uma vez em xxxx
                break;
            case SCHEDULE_REPEAT.HOURLY:
                _msg.push("Rega de hora a hora");  //"hourly",  //A. rega a cada x horas
                break;
            case SCHEDULE_REPEAT.DAILY:
                _msg.push("Rega todos os dias");  // "DAILY",  //A. rega diariamente
                break;
            case SCHEDULE_REPEAT.SPECIFIC_WEEKDAY:
                let dias = self.repeat_spec_wd.split("|");
                let abrev = dias.map((item)=>{return DAYS_ABREV[item]});
                let nr_dias = dias.length;
                _msg.push("Rega ");    // "specific_weekday", //A. rega as (S, T, ... e D) C
                let interjeicao = "à ";
                if (nr_dias === 1 ){
                    if (WEEK_DAYS.SATURDAY in dias || WEEK_DAYS.SUNDAY in dias){
                        interjeicao = "ao "
                    }
                }else{
                    interjeicao = "às "
                }
                _msg.push(interjeicao);
                let sdias = abrev.reduce((acc, item, index, array)=>{if (index === 0){return item;} else {return `${acc},${item}`;} });
                _msg.push(sdias);
                break;
            case SCHEDULE_REPEAT.WEEKLY:
                _msg.push("Rega semanalmente");  //"weekly",  //A. rega semanalmente
                break;
            case SCHEDULE_REPEAT.EVERY:
                _msg.push(`Rega a cada ${self.repeat_every_qty} ${PT_UNITS[this.repeat_every_unit]}`);
                if (this.repeat_every_qty > 1){
                    _msg.push("s"); // "every"   //A. rega a cada
                }
                break
        }
        switch(this.stop_condition){// never, x-retries, date
            case SCHEDULE_STOP.NEVER: // "never",
                break;
            case SCHEDULE_STOP.RETRIES: // "retries",  //B. ____ durante y vezes
                _msg.push(` durante ${this.repeat_every_unit} vezes `);
                break;
            case SCHEDULE_STOP.DATE:   // "date"     // B. _____ até y
                _msg.push(` até ${moment.unix(this.stop_date_ts).format()}`);
                break
        }
        return _msg.join("");
    }
}

//TESTED
function Sector(){
    Row.call(this);
    this.prototype = Object.create(Row.prototype);

    Object.defineProperty(this.prototype, 'constructor', {value: this, enumerable: false, writable: true });
    this.last_change = 0;
    this.op = "I";

    this.id = -1;
    this.description = "";
    this.short_description = "";
    this.week_acc = 0.0;
    this.precipitation = 0.0;  //# mm / minuto
    this.debit = 0.0;  //# mm / minuto
    this.last_watered_in = 0; //last_watered_in

    this.enabled = true;
    this.duration = 20.0;

    this.start_utc_ts = 0;
    this.end_utc_ts = 0;
    this.minutes_to_water = 0.0;
    this.status = 0;

    this.watering_percent = 0.0;

    this.stress_score = 0;
    this.stress_perc = 0;

    // noinspection JSUnusedGlobalSymbols
    this.ins = function(id,
                        description,
                        week_acc,
                        precipitation,
                        debit,
                        last_watered_in,
                        enabled,
                        duration,
                        start_utc_ts,
                        end_utc_ts,
                        minutes_to_water,
                        status,
                        last_change,
                        op){
        this.insFO({id               :id,
                    description      :description,
                    week_acc         :week_acc,
                    precipitation    :precipitation,
                    debit            :debit,
                    last_watered_in  :last_watered_in,
                    enabled          :enabled,
                    duration         :duration,
                    start_utc_ts     :start_utc_ts,
                    end_utc_ts       :end_utc_ts,
                    minutes_to_water :minutes_to_water,
                    status           :status,
                    last_change      :last_change,
                    op               :op})
    };

    // noinspection JSUnusedGlobalSymbols
    this.last_watered_in_str = function(){
        return moment.unix(this.last_watered_in).format(TIME_24H_FORMAT);
    };
    // noinspection JSUnusedGlobalSymbols
    this.enabled_str = function(){
        return (this.enabled)?"Operacional":"Manutenção";
    };
    this.status_str = function(){
        return (this.status === WATERING_STATUS.RUNNING)?"ligado":"desligado"
    };
    this.last_watered_in_full_str = function(){
        if (this.last_watered_in === 0){
            return "N/A";
        }else{
            return moment.unix(this.last_watered_in).format(DATE_LONG_FORMAT);
        }
    };
    this.end_utc_ts_str = function(){
        return moment.unix(this.end_utc_ts).format(DATE_LONG_FORMAT);
    };
    this.start_utc_ts_str = function(){
        let result = "";
        if (!this.enabled){
            result = "N/A";
        }else{
            result = moment.unix(this.start_utc_ts).format(TIME_24H_FORMAT);
        }
        return result;
    };
    this.end_hour_ts_str = function(){
        return moment.unix(this.end_utc_ts).format(TIME_24H_FORMAT);
    };
    this.watering_percent_str = function(){
        return round((100 * this.watering_percent),0).toString();
    };

    this.stress_perc_str = function(){
        return Math.min(999, round(this.stress_perc,0)); 
    };
}