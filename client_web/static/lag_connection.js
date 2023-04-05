"use strict";

var controller_connection = function(){
    this.nothing_connected_ui = document.getElementById("nothing_connected");
    this.mqtt_connected_ui = document.getElementById("mqtt_connected");
    this.server_connected_ui = document.getElementById("server_connected");
    this.all_connected_ui = document.getElementById("all_connected");

    this.connection_status = CONNECTION_STATUS.NOTHING;
    this.connected_event = new LagEvent();
    this.disconnected_event = new LagEvent();

    this.mqtt_connected = function(){
        switch(this.connection_status){
            case CONNECTION_STATUS.NOTHING:
                hide(this.nothing_connected_ui);
                show(this.mqtt_connected_ui);
                this.connection_status = CONNECTION_STATUS.MQTT;
                break;
            case CONNECTION_STATUS.SERVER:
                hide(this.server_connected_ui);
                show(this.all_connected_ui);
                this.connection_status = CONNECTION_STATUS.ALL;
                this.connected_event.notifyObservers(null);
                break;
        }
    }.bind(this);

    this.server_connected = function(){
        switch(this.connection_status){
            case CONNECTION_STATUS.NOTHING:
                hide(this.nothing_connected_ui);
                show(this.server_connected_ui);
                this.connection_status = CONNECTION_STATUS.SERVER;
                break;
            case CONNECTION_STATUS.MQTT:
                hide(this.mqtt_connected_ui);
                show(this.all_connected_ui);
                this.connection_status = CONNECTION_STATUS.ALL;
                this.connected_event.notifyObservers(null);
                break;
        }
    }.bind(this);

    this.mqtt_disconnected = function(){
        switch(this.connection_status){
            case CONNECTION_STATUS.MQTT:
                hide(this.mqtt_connected_ui);
                show(this.nothing_connected_ui);
                this.connection_status = CONNECTION_STATUS.NOTHING;
                break;
            case CONNECTION_STATUS.ALL:
                hide(this.all_connected_ui);
                show(this.server_connected_ui);
                this.connection_status = CONNECTION_STATUS.SERVER;
                this.disconnected_event.notifyObservers(null);
                break;
        }
    }.bind(this);

    this.server_disconnected = function(){
        switch(this.connection_status){
            case CONNECTION_STATUS.SERVER:
                hide(this.server_connected_ui);
                show(this.nothing_connected_ui);
                this.connection_status = CONNECTION_STATUS.NOTHING;
                break;
            case CONNECTION_STATUS.ALL:
                hide(this.all_connected_ui);
                show(this.mqtt_connected_ui);
                this.connection_status = CONNECTION_STATUS.MQTT;
                this.disconnected_event.notifyObservers(null);
                break;
        }
    }.bind(this);

    this.is_everything_connected = function(){
        return this.connection_status === CONNECTION_STATUS.ALL;
    }.bind(this);

    this.is_mqtt_connected = function(){
        return this.connection_status === CONNECTION_STATUS.MQTT || this.connection_status === CONNECTION_STATUS.ALL;
    }.bind(this);

    this._connected_event = function (param) {
        this.mqtt_connected();
    }.bind(this);

    this._disconnected_event = function (param) {
        this.mqtt_disconnected();
    }.bind(this);

    this.register_client_sync = function(request){
        let response = request.responseText;
        let self = this;
        
        if (request.status === 200) {
            CTRL_CONN.server_connected();
            config_client_id(JSON.parse(response).id);
            self.register_client_backoff = 1;
            clearTimeout(self.register_timeout);
            if (!MQTT.isConnected()){
                MQTT.start();
            }
        }else{
            this.server_disconnected();
            // increments time between retries, up to 1 minute, and every 1 minute afterwords
            self.register_client_backoff = Math.min(self.register_client_backoff * 2 , 60);
            self.register_timeout = setTimeout(REST_API.call_server_sync, self.register_client_backoff * 1000, self.url, self.register_client_sync);
        };
        self = null;
    }.bind(this);

    this.register_client_backoff = 1;
    this.register_timeout = null;
    this.url = REST_API.build_url(APP_CONTROLLER, CMD_ID);
}
