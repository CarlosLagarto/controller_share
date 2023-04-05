"use strict";

// TESTED
function controllerRega(){

    this.is_manual = function(){
        return this.model.mode === WATER_MACHINE_MODE.MANUAL; 
    }.bind(this);

    this.is_not_manual = function(){
        return this.model.mode !== WATER_MACHINE_MODE.MANUAL;
    }.bind(this);

    this._sync_VM_status = function (){
        this.model.update_status(DB.config);
        this.view.changingState = false;
    }.bind(this);

    this.notify_new_data = function(newDataObj){
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
                    this.view.render_status();
                    break;
            }
            MAIN.render_date(get_js_now_time_adjusted());
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    this.notifyConnection = function(config){
        let self = this;
        try{
            // order is important due to logic to handle mode
            self._sync_VM_status();
            this.view.render_status();
            self.view.render_cycles_V1();
            self.view.render_sectors_V1();
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    this.alertArrived = function(alert_message){
        try{
            this.model.alert = alert_message.msg.type_;
            let msg = alert_message.msg;
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(msg, msg.header.uuid));
            this.view.render_status();
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    this.alertResetArrived = function(alert_message){
        try{
            this.model.alert = ALERT_TYPE.NoAlert;
            let msg = alert_message.msg;
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(msg, msg.header.uuid));
            this.view.render_status();
        }catch(err){
            log.error(err);
        }
    }.bind(this);

    this.model = new modelRega();
    this.model.update_status(DB.config);
    this.view = new viewRega(this);

    try{
        this.view.cyclesBuild();
        this.view.sectorsBuild();
        this._sync_VM_status();
        DB.newDataEvent.registerObserver(this, this.notify_new_data);
        
        MQTT.onSTCAlertArrived = this.alertArrived;
        MQTT.onSTCAlertResetArrived = this.alertResetArrived;
    }catch(err){
        log.error(err);
    }
}
