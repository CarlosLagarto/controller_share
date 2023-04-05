"use strict";

// Sensors and scenes - CENAS - always handled by the REST API.

function db () {
    // implements a basic storage on top of local storage.
    // backend database table colmuns and properties in the client have to match

    //TESTED
    this.isCycleDefined = function(id_ciclo){
        return this.cycles.hasOwnProperty(id_ciclo);
    }.bind(this);

    this.get_next_cycle_id = function(){
        let self = this;
        let max = Number.MAX_VALUE;
        let aux_id_max="";
        let cycle = null;

        for (let id in self.cycles){
            cycle = self.cycles[id];
            if (!cycle.is_running()){
                switch(self.config.mode){
                    case WATER_MACHINE_MODE.STANDARD:
                        if (cycle.cycle_type === CYCLE_TYPE_INDEX.STANDARD){
                            if (cycle.start < max){
                                aux_id_max = id;
                                max = cycle.start;
                            }            
                        }
                        break;
                    case WATER_MACHINE_MODE.WIZARD:
                        if (cycle.cycle_type === CYCLE_TYPE_INDEX.WIZARD){
                            if (cycle.start < max){
                                aux_id_max = id;
                                max = cycle.start;
                            }            
                        }
                        break;
                    case WATER_MACHINE_MODE.MANUAL:
                        break;
                }
            }else{
                aux_id_max = id;
                break;
            }
        }
        self.next_cycle_id = aux_id_max;
        self = null;
        max = null;
        aux_id_max = null;
        cycle = null;
    }.bind(this);

    //TESTED
    this.requestFullsyncFromServer = function (){
        let self = this;
        try{
            self.syncType = SYNC_TYPE.FULL;
            self.db_requested = true;
            let msg1 = {};
            buildAndSendMessage(MSG.T.CTS_GET_FULLDB, msg1);
        }
        catch(err){
            log.error(err)
        }
        self = null;
    }.bind(this);

    //TESTED
    this.connected_event = function(param){
        this.requestFullsyncFromServer(); 
    }.bind(this);

    //TESTED
    this._populate = function(localStorage_tbl, obj_class, cli_tbl){
        let str_tbl = localStorage.getItem(localStorage_tbl);
        let tbl = JSON.parse(str_tbl);
        let c =  null;
        let io = null;
        for (let row_id in tbl){
            c = new obj_class();
            io = tbl[row_id];
            c.set(io);  
            cli_tbl[row_id] = c;
        }
        c = null;
        io = null;
        tbl = null;
        str_tbl = null;
    }.bind(this);

    //TESTED
    this._populateRow = function(row_id, cli_row){
        let str_row = localStorage.getItem(row_id);
        let row = JSON.parse(str_row);
        cli_row.conditionalSet(row);  
        str_row = null;
        row = null;
    }.bind(this);

    //TESTED
    this.__notifyObserver = function(tbl_data, tbl_name, forceRefresh){
        if(Object.keys(tbl_data).length > 0 || forceRefresh){
            this.newDataEvent.notifyObservers({type:tbl_name,data: tbl_data});
        }
    }.bind(this);

    //TESTED
    this.__notifyObservers = function(last_sync, forceRefresh){
        if (last_sync > 0 || forceRefresh){
            let self = this;
            // config should be first table due to the influence that the mode parameter have in the remaining logic
            self.__notifyObserver(self.config, "config", forceRefresh);
            self.__notifyObserver(self.cycles, "cycles", forceRefresh);
            self.__notifyObserver(self.sectors, "sectors", forceRefresh);
            self = null;
        }
    }.bind(this);

    //TESTED
    this.open = function(){
        // get basic info from storage
        let last_sync = localStorage.getItem(BD_LAST_SYNC);
        let self = this;

        if ( last_sync === "undefined" || last_sync === null){  // storage still lnot defined so lets create it
            self.save();
        }
        self.last_sync = parseInt(last_sync);
        // show last saved info on start up
        try{
            self._populate(BD_CYCLES_TBL, Cycle, self.cycles);
            self._populate(BD_SECTORS_TBL, Sector, self.sectors);
            self._populateRow(BD_CONFIG_ROW, self.config);
            self.get_next_cycle_id();
            self.__notifyObservers(self.last_sync, true);
        }
        catch(error){
            error.message = "Erro a abrir a base de dados: " + error.message;
            log.error(error);
        }
        self = null;
        last_sync = null;
    }.bind(this);

    //TESTED
    this.save = function(){
        let self = this;

        localStorage.setItem(BD_LAST_SYNC, self.last_sync);
        localStorage.setItem(BD_CYCLES_TBL, JSON.stringify(self.cycles));
        localStorage.setItem(BD_SECTORS_TBL, JSON.stringify(self.sectors));
        localStorage.setItem(BD_CONFIG_ROW, JSON.stringify(self.config))

        self.get_next_cycle_id();
        self = null;
    }.bind(this);

    //TESTED
    this._ins = function(no, io, table, pk){  
        no.insFO(io);  
        table[pk] = no;
    }.bind(this);
    //TESTED
    this._upd = function(io, table, pk){  
        let no = table[pk];
        return no.upd(io);   // returns changed columns
    }.bind(this);
    //TESTED
    this._upd_row = function(io, cli_row){  
        return cli_row.upd(io);  // returns changed columns
    }.bind(this);
    //TESTED
    this._del= function(table, pk){
        delete table[pk];
    }.bind(this);

    //TESTED
    this._sync_table = function(srv_tbl, cli_tbl, obj_class, id_field){
        let io = null;
        let pk = null;
        let self = this;
        let cli_obj = null;
        for (let row_id in srv_tbl){
            io = srv_tbl[row_id];
            pk = io[id_field];
            cli_obj = cli_tbl[pk];
            if (cli_obj){
                if (io.op === OP.U){ //TESTED
                    self._upd(io, cli_tbl, pk);
                    if (io.last_change < cli_obj.last_change){
                        // changes in the client after the backend changes
                        // assume client is the master, but give an alert
                        log.addEntry(LOG_ENTRY_TYPE.LOG,  log.newEntry("tema sync - no " + srv_tbl + " no update." + row_id));
                    }

                } else {
                    if (io.op === OP.D){ //NOTTESTED
                        self._del(cli_tbl, io[pk]);

                        if (io.last_change < cli_obj.last_change){
                            // changes in the client after the backend changes
                            // assume client is the master, but give an alert
                            log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry("tema sync - no " + srv_tbl + " no delete." + row_id));
                        }
                    } else {
                        if (io.op === OP.I){ //TESTED
                            self._ins(new obj_class(), io, cli_tbl, pk);
                        }else{  
                            log.addEntry(LOG_ENTRY_TYPE.LOG,  log.newEntry("operação de sync desconhecida: " + io.op + ", " + row_id));
                        }
                    }
                }
            }else{
                // happens on startup (no records yet) or someone made a cleansing
                // create the record
                self._ins(new obj_class(), io, cli_tbl, pk);
            }
        }
        // DESIGN NOTE 1: we are syncing server info, and client deletd records only would be deleted avter a full sync with the server
        for (let row_id in cli_tbl){
            cli_obj = cli_tbl[row_id];
            if (cli_obj.op === OP.D){ 
                self._del(cli_tbl, cli_obj[id_field]);
            }
        }
        io = null;
        pk = null;
        cli_obj = null;
        self = null;
    }.bind(this);

    //TESTED
    this._sync_row = function(srv_row, cli_row){
        this._upd_row(srv_row, cli_row);
    }.bind(this);

     //TESTED
    this._fillClientTable = function(clientClass, cli_tbl, srv_tbl, pk){
        let io = null;
        let self = this;
        let real_id = null;
        for (let id in srv_tbl){    //{ rowid1 : {...}, ..., rowidn : {...}, ...}
            real_id = srv_tbl[id][pk];
            io = srv_tbl[id];
            self._ins(new clientClass(), io, cli_tbl, real_id);
        }
        io = null;
        self = null;
        real_id = null;
    }.bind(this);

    //TESTED
    this.syncFromServer = function(message){
        let msg = null;
        let self = this;

        if (self.db_requested){
            self.db_requested = false;  // info received
            CTRL_CONN.connected_event.unregisterObserver(this);
        }
        try{
            msg = message.msg.db_sync;

            let syncType = msg.sync_type;
            self.last_sync = msg.last_sync;
            self._sync_row(msg.config, self.config);
            if (syncType === SYNC_TYPE.FULL){
                // clean tables
                self.cycles = {};
                self.sectors = {};
                // sensors is by REST API
                // load with new data
                self._fillClientTable(Cycle, self.cycles, msg.cycles, "cycle_id");
                self._fillClientTable(Sector, self.sectors, msg.sectors, "id");
                self.syncType = SYNC_TYPE.PARTIAL;  // next update will be partial
            }else{ // partial update
                // new cycles leaving the client have an id of 0.  The right id it's only assigned by the server
                // so we assume that server info is always right, so we can dismiss the cycles with ID == 0, as well as the deleted ones.
                // not a very bright algorithm performance wise (do not scale) but when talking about less then 10 records, it's a esteril discussion
                for(let id in msg.cycles){
                    if(id == 0) delete msg.cycles[id];
                }
                for(let id in self.cycles){
                    if(self.cycles[id].op === OP.D) delete self.cycles[id];
                }
                self._sync_table(msg.cycles, self.cycles, Cycle, "cycle_id");
                self._sync_table(msg.sectors, self.sectors, Sector, "id");
                // CENAS is handled by REST API or MQTT status update
            }
            self.save();
            self.__notifyObservers(self.last_sync, true);
        } catch(err){
            log.error(err);
        } finally{
            msg = null;
            self = null;
        }
    }.bind(this);

    //TESTED
    this.__getRow = function(row_dest, row_origin, attributes){
        for(let attr of attributes)
            row_dest[attr] = row_origin[attr];
    }.bind(this);
    
    //TESTED
    this.__getTableMsg = function(last_sync, tbl_to_send, srv_tbl, attributes){
        let row = null;
        let self = this;
        for (let row_id in srv_tbl){
            row = srv_tbl[row_id];
            // verify the updates for each record last_change > this.last_sync
            if (row.last_change > last_sync){
                let new_row = {};
                self.__getRow(new_row, row, attributes);
                tbl_to_send.push(new_row);
            }
        }
        row = null;
        self = null;
        return tbl_to_send;
    }.bind(this);

    //TESTED
    this.syncToServer = function(){
        let msg = {db_sync: {}};
        let self = this;
        msg.db_sync["sync_type"] = SYNC_TYPE.PARTIAL;
        msg.db_sync["last_sync"] = get_unix_now_time_adjusted();
        msg.db_sync["cycles"] = [];
        msg.db_sync["sectors"] = [];
        
        self.__getTableMsg(self.last_sync, msg.db_sync["cycles"], self.cycles, CYCLE_ATTRIBUTES);
        self.__getTableMsg(self.last_sync, msg.db_sync["sectors"], self.sectors, SECTOR_ATTRIBUTES);
        
        if (self.config.last_change > self.last_sync){
            msg.db_sync["config"] = {};
            self.__getRow(msg.config, self.config, CONFIG_ATTRIBUTES_UPDATABLE);
        }
        buildAndSendMessage(MSG.T.CTS_SYNC_DB, msg);
        msg = null;
        self = null;        
    }.bind(this);

    //TESTED
    this._notifyObservers = function (srv_obj, cli_obj){
        if(Object.keys(srv_obj).length  > 0){
            this.newDataEvent.notifyObservers(cli_obj);
        }
    }.bind(this);

    //TESTED
    this.addCycle = function(cycle){
        let self = this;
        cycle.last_change = get_unix_now_time_adjusted();
        cycle.op = OP.I;
        self._ins(new Cycle(), cycle, self.cycles, cycle.cycle_id);
        self.save();
        self = null;
    }.bind(this);

    //TESTED
    this.updateCycle = function(cycle){
        let self = null;
        cycle.last_change = get_unix_now_time_adjusted();
        cycle.op = OP.U;
        self._upd(cycle, self.cycles, cycle.cycle_id);
        self.save();
        self = null;
    }.bind(this);

    //TESTED
    this.updateSector = function(sector){
        let self = this;
        sector.last_change = get_unix_now_time_adjusted();
        sector.op = OP.U;
        self._upd(sector, self.sectors, sector.id);
        self.save();
        self = null;
    }.bind(this);

    //TESTED
    this.delCycle = function(cycle){
        let self = this;
        let _cycle = self.cycles[cycle.cycle_id];
        _cycle.op = OP.D;
        _cycle.last_change = get_unix_now_time_adjusted();
        self.save();
        cycle = null;
        self = null;
    }.bind(this);

    this.db_requested = false;
    this.last_sync = 0;

    this.cycles = {};  
    this.sectors = {}; 
    this.config = new Config();
    this.syncType = SYNC_TYPE.UNDEFINED;
    this.next_cycle_id = "";

    this.newDataEvent = new LagEvent();
}
