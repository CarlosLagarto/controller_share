"use strict";

//TESTED
function Row() {

    this.set = function (row) {
        let self = this;
        for (let attr in row) {
            if (typeof (row[attr]) !== "undefined" && typeof (row[attr]) !== "function" && typeof (row[attr]) !== "object") {
                self[attr] = row[attr];
            }
        }
        self = null;
    }.bind(this);

    this.conditionalSet = function (row) {
        let changedFields = [];
        let self = this;
        for (let attr in row) {
            if (typeof (row[attr]) !== "undefined" && typeof (row[attr]) !== "function" && typeof (row[attr]) !== "object") {
                if (self[attr] !== row[attr]) {
                    self[attr] = row[attr];
                    changedFields.push(attr);
                }
            }
        }
        self = null;
        return changedFields;
    }.bind(this);

    this.insFO = function (obj) {
        this.set(obj);
    }.bind(this);

    this.upd = function (row) {
        return this.conditionalSet(row);
    }.bind(this);
}

//TESTED
function Config() {
    Row.call(this);
    this.prototype = Object.create(Row.prototype)
    Object.defineProperty(this.prototype, 'constructor', { value: this, enumerable: false, writable: true });

    this.last_sync = 0;
    this.last_change = 0;//ok

    this.mode = "";//ok
    this.in_error = 0;//ok
    this.in_alert = ALERT_TYPE.NoAlert;//ok
    this.date = "";

    this.max_sector_time = 0;//ok
    this.pump_recycle_time = 0;//ok
    this.schedule_def = 0;
    this.rain_alert_threshold = 0;//ok
    this.wind_alert_threshold = 0;//ok
    this.watering_suspend_timeout = 0;//ok
    this.decrease_alert_level_after = 0;//ok
    this.stress_control_interval = 0;//ok

    this.db_maint_days = 0;//ok
    this.db_maint_counter = 0;

    this.live_since = "";//ok
    this.current_state = "";//ok
    this.simulation = 0;//ok

    this.ins = function (mode,
        in_error,
        in_alert,
        date,
        max_sector_time,
        pump_recycle_time,
        schedule_def,
        rain_alert_threshold,
        wind_alert_threshold,
        watering_suspend_timeout,
        decrease_alert_level_after,
        stress_control_interval,
        db_maint_days,
        db_maint_counter,
        live_since,
        current_state,
        simulation,
        last_sync,
        last_change) {
        this.insFO({
            mode: mode,
            in_error: in_error,
            in_alert: in_alert,
            date: date,
            max_sector_time: max_sector_time,
            pump_recycle_time: pump_recycle_time,
            schedule_def: schedule_def,
            rain_alert_threshold: rain_alert_threshold,
            wind_alert_threshold: wind_alert_threshold,
            watering_suspend_timeout: watering_suspend_timeout,
            decrease_alert_level_after: decrease_alert_level_after,
            stress_control_interval: stress_control_interval,
            db_maint_days: db_maint_days,
            db_maint_counter: db_maint_counter,
            live_since: live_since,
            current_state: current_state,
            simulation: simulation,
            last_sync: last_sync,
            last_change: last_change
        })
    }.bind(this);

    this.clone = function () {
        let o = new Config();
        o.set({
            mode: this.mode,
            in_error: this.in_error,
            in_alert: this.in_alert,
            date: this.date,
            max_sector_time: this.max_sector_time,
            pump_recycle_time: this.pump_recycle_time,
            schedule_def: this.schedule_def,
            rain_alert_threshold: this.rain_alert_threshold,
            wind_alert_threshold: this.wind_alert_threshold,
            watering_suspend_timeout: this.watering_suspend_timeout,
            decrease_alert_level_after: this.decrease_alert_level_after,
            stress_control_interval: this.stress_control_interval,
            db_maint_days: this.db_maint_days,
            db_maint_counter: this.db_maint_counter,
            live_since: this.live_since,
            current_state: this.current_state,
            simulation: this.simulation,
            last_sync: this.last_sync,
            op: this.op,
            last_change: this.last_change
        });
        return o;
    }.bind(this);
}

//TESTED
function Cycle() {
    Row.call(this);
    this.prototype = Object.create(Row.prototype)

    Object.defineProperty(this.prototype, 'constructor', { value: this, enumerable: false, writable: true });

    this.start = 0; //visto
    this.end = 0; //visto
    this.cycle_id = 0;  //visto
    this.run_id = 0; //visto
    this.status = ""; //visto
    this.run_start = 0; //visto
    this.repeat_kind = "";  //# never/hourly/daily/weekday/specific week days/weekly/monthly/every //visto
    this.stop_condition = ""; //  # "never", #never, x-retries, date  //visto
    this.stop_retries = 0; //visto
    this.stop_date_ts = 0; //  # 0 #timestamp  //visto
    this.repeat_spec_wd = 0; //  Agora tem a codificação num u8 # "", #Su|Mo|Tu|We|Th|Fr|St  //visto 
    this.repeat_every_qty = 0;  //visto
    this.repeat_every_unit = ""; //  # "", #minutes, hours, days, week, month  //visto
    this.retries_count = 0; //visto
    this.name = "";  //visto
    this.last_change = 0;  //visto
    this.last_run = 0;  //visto
    this.sim = 0; //ok
    this.op = "I"; //visto
    this.sunrise_flg = 1; //visto
    this.sunset_flg = 0; //visto
    this.cycle_type = 0;

    this.exec_perc = 0.0;

    this.ins = function (start, //ok
        end, //ok
        cycle_id, //ok
        run_id, //ok
        status, //ok
        run_start,//ok
        repeat_kind, //ok
        stop_condition, //ok
        stop_retries, //ok
        stop_date_ts, //ok
        repeat_spec_wd, //ok
        repeat_every_qty, //ok
        repeat_every_unit, //ok
        retries_count, //ok
        name,  //ok
        last_change, //ok
        last_run, //ok
        sim, //ok
        op, //ok
        sunrise_flg, //ok
        sunset_flg, //ok
        cycle_type,
    ) {
        this.insFO({
            start: start, //ok
            end: end, //ok
            cycle_id: cycle_id, //ok
            run_id: run_id, //ok
            status: status, //ok
            run_start: run_start, //ok
            repeat_kind: repeat_kind, //ok
            stop_condition: stop_condition, //ok
            stop_retries: stop_retries, //ok
            stop_date_ts: stop_date_ts, //ok
            repeat_spec_wd: repeat_spec_wd, //ok
            repeat_every_qty: repeat_every_qty,//ok
            repeat_every_unit: repeat_every_unit, //ok
            retries_count: retries_count,  //ok
            name: name, //ok
            last_change: last_change, //ok
            last_run: last_run, //ok
            sim: sim, //ok
            op: op, //ok

            sunrise_flg: sunrise_flg,
            sunset_flg: sunset_flg,
            cycle_type: cycle_type,
        })
    }.bind(this);

    this.start_ts_str = function () {
        return date_to_iso8601(unix_to_js_date(this.start));
    }.bind(this);

    this.name_str = function(machine_status) {
        if (this.cycle_type === CYCLE_TYPE_INDEX.DIRECT){
            switch (machine_status){ 
                case "ManWtrCycle" :
                case "ManWtrSector":{
                    return this.name + " cycle";
                    break;
                }
                case "ManWtrSectorDirect":
                case "Manual: Setor ativo":{
                    return this.name + " sector";
                    break;
                }
                default: {
                    // case ("Starting" ||  "NoScheduleDef" || "EstablishMode" || "ManWait" || "WzrWait" || "StdWait" || "StdWtrCycle" 
                    // || "WzrWtrCycle" || "StdWtrSector" || "WzrWtrSector" || "SuspendedWizard" || "Error" || "Shutdown"):
                    return this.name;
                    // break;    
                }
            }
        }else{
            return this.name;
        }
    }.bind(this);

    this.stop_date_ts_str = function () {
        return date_to_iso8601(unix_to_js_date(this.stop_date_ts)); 
    }.bind(this);

    this.prev_cycle_description = function () {
        if (this.last_run <= get_unix_now_time() - (86400 * 31)) {
            return "á mais de 1 mês"
        } else {
            return date_to_iso8601(unix_to_js_date_adjusted(this.last_run));
        }
    }.bind(this);

    this.next_cycle_description = function () {
        return date_to_iso8601(unix_to_js_date(this.start));
    }.bind(this);

    this.start_str = function () {
        return get_time_str_from_unix(this.start);
    }.bind(this);

    this.end_str = function () {
        return get_time_str_from_unix(this.end);
    }.bind(this);

    this.start_exec_perc_str = function () {
        return round((100 * this.exec_perc), 0).toString();
    }.bind(this);

    //NOTTESTED
    this.desc = function () {
        let _msg = [];
        let self = this;
        switch (this.repeat_kind) {
            case SCHEDULE_REPEAT.NEVER:
                _msg.push(`Regou uma vez em ${date_to_iso8601(unix_to_js_date(self.start))}`);  //A. regou uma vez em xxxx
                break;
            case SCHEDULE_REPEAT.HOURLY:
                _msg.push("Rega de hora a hora");  //"hourly",  //A. water each x hours
                break;
            case SCHEDULE_REPEAT.DAILY:
                _msg.push("Rega todos os dias");  // "DAILY",  //A. daily water
                break;
            case SCHEDULE_REPEAT.SPECIFIC_WEEKDAY:
                let abrev = []; 
                let nr_dias = 0;
                // start from left bit 
                let a = 128;
                for (let i = 7; i >= 0; i--) {
                    // shift right
                    a >>= 1;
                    if (self.repeat_spec_wd & a) {
                        nr_dias += 1;
                        abrev.push(DAYS_ABREV_IDX[i]);
                    }
                };

                _msg.push("Rega ");    // "specific_weekday", //A. specific day watering  (S, T, ... e D) 
                let interjeicao = "à ";
                if (nr_dias === 1) {
                    if (WEEK_DAYS_ABR.SATURDAY in abrev || WEEK_DAYS_ABR.SUNDAY in abrev) {
                        interjeicao = "ao "
                    }
                } else {
                    interjeicao = "às "
                }
                _msg.push(interjeicao);
                let sdias = abrev.reduce((acc, item, index, array) => { if (index === 0) { return item; } else { return `${acc},${item}`; } });
                _msg.push(sdias);
                break;
            case SCHEDULE_REPEAT.WEEKLY:
                _msg.push("Rega semanalmente");  //"weekly",  //A. water weekly
                break;
            case SCHEDULE_REPEAT.EVERY:
                _msg.push(`Rega a cada ${self.repeat_every_qty} ${PT_UNITS[this.repeat_every_unit]}`);
                if (this.repeat_every_qty > 1) {
                    _msg.push("s"); // "every"   //A. water each...
                }
                break
        }
        switch (this.stop_condition) {// never, x-retries, date
            case SCHEDULE_STOP.NEVER: // "never",
                break;
            case SCHEDULE_STOP.RETRIES: // "retries",  //B. ____ for y times
                if (this.repeat_kind !== SCHEDULE_REPEAT.NEVER) {
                    _msg.push(` durante ${this.repeat_every_qty} vezes `);
                }
                break;
            case SCHEDULE_STOP.DATE:   // "date"     // B. _____ until y
                _msg.push(` até ${date_to_iso8601(unix_to_js_date(this.stop_date_ts))}`);
                break
        }
        self = null;
        return _msg.join("");
    }.bind(this);

    this.is_running = function () {return this.status === WATERING_STATUS.RUNNING};
}

//TESTED
function Sector() {
    Row.call(this);
    this.prototype = Object.create(Row.prototype);

    Object.defineProperty(this.prototype, 'constructor', { value: this, enumerable: false, writable: true });

    this.desc = ""; //visto
    this.name = ""; //visto
    this.last_watered_in = 0; //last_watered_in //visto
    this.last_change = 0; //visto
    this.deficit = 0.0; //visto
    this.percolation = 0.0;  //# mm / minuto //visto
    this.debit = 0.0;  //# mm / minuto //visto
    this.max_duration = 20.0; //visto
    this.stress_perc = 0;  //visto
    this.stress_score = 0; //visto
    this.id = -1; //visto
    this.enabled = true;  //visto
    this.op = "I"; //visto

    this.start = 0;
    this.end = 0;
    this.minutes_to_water = 0.0;
    this.status = 0;

    this.watering_percent = 0.0;

    this.ins = function (desc, //ok
        name, //ok
        last_watered_in, //ok
        last_change, //ok
        deficit, //ok
        percolation, //ok
        debit, //ok
        max_duration, //ok
        stress_perc, //ok
        stress_score, //ok
        id, //ok
        enabled, //ok
        op, //ok
        start,
        end,
        minutes_to_water,
        status
    ) {
        this.insFO({
            desc: desc, //ok
            name: name, //ok
            last_watered_in: last_watered_in, //ok
            last_change: last_change, //ok
            deficit: deficit, //ok
            percolation: percolation, //ok
            debit: debit, //ok
            max_duration: max_duration, //ok
            stress_perc: stress_perc, //ok
            stress_score: stress_score, //ok
            id: id, //ok
            enabled: enabled, //ok
            op: op, //ok

            start: start,
            end: end,
            minutes_to_water: minutes_to_water,
            status: status,
        })
    }.bind(this);

    this.last_watered_in_str = function () {
        return get_time_str_from_unix(this.last_watered_in);
    }.bind(this);

    this.enabled_str = function () {
        return (this.enabled) ? "Operacional" : "Manutenção";
    }.bind(this);

    this.status_str = function () {
        return (this.is_running()) ? "ligado" : "desligado"
    }.bind(this);

    this.last_watered_in_full_str = function () {
        if (this.last_watered_in === 0) {
            return "N/A";
        } else {
            return date_to_iso8601(unix_to_js_date(this.last_watered_in));
        }
    }.bind(this);

    this.end_str = function () {
        return date_to_iso8601(unix_to_js_date(this.end));
    }.bind(this);

    this.start_str = function () {
        let result = "";
        if (!this.enabled) {
            result = "N/A";
        } else {
            result = get_time_str_from_unix(this.start);
        }
        return result;
    }.bind(this);

    this.end_hour_ts_str = function () {
        return get_time_str_from_unix(this.end);
    }.bind(this);
    
    this.watering_percent_str = function () {
        return round((100 * this.watering_percent), 0).toString();
    }.bind(this);

    this.stress_perc_str = function () {
        return clamp(round(this.stress_perc, 0), 0, 100);
    }.bind(this);

    this.is_waiting = function () {
        return this.status === WATERING_STATUS.WAITING;
    }.bind(this);

    this.is_running = function () {
        return this.status === WATERING_STATUS.RUNNING;
    }.bind(this);
}

//TESTED
function modelRega() {
    this.date = undefined;
    this.machine_status = "sem informação"; //0 or 1, on or off
    this.error = 0; //0 ou 1
    this.alert = ALERT_TYPE.NoAlert; 
    this.mode = WATER_MACHINE_MODE.MANUAL; //manual, standard or wizard

    this.update_status = function(config){
        let self = this;
        self.machine_status = config.current_state;
        self.error = config.in_error;
        self.alert = config.in_alert;
        self.mode = config.mode;
    }.bind(this);
}
