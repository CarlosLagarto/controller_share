"use strict";

//NOTTESTED
function modelRega() {

    this.date = undefined;
    this.machine_status = "sem informação"; //0 ou 1, on ou off
    this.error = 0; //0 ou 1
    this.alert = 0; //0 ou 1
    this.mode = "manual"; //manual, standard ou wizard

    this.update_status = function(config){
        let self = this;
        //this.date = status.date;
        self.machine_status = config.current_state;
        self.error = config.in_error;
        self.alert = config.in_alert;
        self.mode = config.mode;
    }
}

// TESTED
function controllerRega(){
    this.model = new modelRega();
    this.model.update_status(DB.config);
    this.view = new viewRega(this);

    this._sync_VM_status = function (){
        this.model.update_status(DB.config);
        this.view.changingState = false;
        this.view.render_status();
    };

    this.notifyNewData = function(newDataObj){
        try{
            switch(newDataObj.type){
                case "cycles":
                    this.view.check_running_things();
                    this.view.render_cycles_V1();
                    break;
                case "sectors":
                    this.view.check_running_things();
                    this.view.render_sectors_V1();
                    break;
                case "config":
                    this._sync_VM_status();
                    break;
            }
            MAIN.updateDate(moment().format())
        }catch(err){
            log.error(err)
        }
    }.bind(this);

    this.notifyConnection = function(config){
        let self = this;
        try{
            // a ordem é importante por causa do mode
            self._sync_VM_status();
            self.view.render_cycles_V1();
            self.view.render_sectors_V1();
        }catch(err){
            log.error(err);
        }
    };

    this.alertArrived = function(alert_message){
        try{
            this.model.alert = 1;
            let msg = alert_message.body;
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(msg, msg.uuid));
            this.view.render_status();
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    this.alertResetArrived = function(alert_message){
        try{
            this.model.alert = 0;
            let msg = alert_message.body;
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(msg, msg.uuid));
            this.view.render_status();
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    try{
        this.view.cyclesBuild();
        this.view.sectorsBuild();
        this._sync_VM_status();
        DB.newDataEvent.registerObserver(this);
        MQTT.connectionEvent.registerObserver(this);
        MQTT.onSTCAlertArrived = this.alertArrived;
        MQTT.onSTCAlertResetArrived = this.alertResetArrived;
    }catch(err){
        log.error(err);
    }
}
