"use strict";
//#userName password willMessage    invocationContext   hosts ports mqttVersion
/*if ("serviceWorker" in navigator) {
    navigator.serviceWorker.register("service-worker.js")
        .then(function (registration) {
            console.log("Service Worker registered with scope:",
                         registration.scope);
        }).catch(function (err) {
        console.log("Service worker registration failed:", err);
    });
}*/

var CONFIG = null;
var WEATHER = null;
var MSG = null;
var log =  null;
var MQTT = null;
var DB = null;
var CTRL_REGA = null;
var MAIN = null;
var MAIN_VIEW = null;
var REST_API = null;
var DELTA_TIME_MS = 0;
var CENAS = null;

// "feature toggle" - search for "tst" in the url - if exists, means test environment
var TEST = null;
// var LOCATION = window.location;

var CLIENT_ID = "uuid:" + new UUID(1).format();
var CTRL_CONN = null;

var main = function(){

    this.render_date = function(date){
        let suffix = " PRD";
        if (is_test_env()){
            suffix = " TST";
        }
        infoDate.innerHTML = date_to_iso8601(date) + suffix;
    };

    this.connected = function(date){
        this.render_date(date);
        CTRL_CONN.mqtt_connected();
    };

    this.disconnected = function (){
        CTRL_CONN.mqtt_disconnected();
    };

    this.notifyConnection = function(status){
        if (status){
            // let d = (new Date()).toISOString();
            this.connected(get_js_now_time_adjusted());
        } else {
            this.disconnected();
        }
    }.bind(this);

    try{
        MSG = new _MSG();
        // this.controller_connection = new controller_connection();
        this.infoDate = document.getElementById("infoDate");

        MAIN_VIEW = new main_view();
        MAIN_VIEW.ini_application_ui();

        MAIN_VIEW.StatusMsg.setTemp("A ligar e a obter info.", 5);

        log = new logs(); 
        log.populate();
        log.render(null, true);
        window.addEventListener('error', log.error);

        DB = new db();
        DB.open();

        REST_API = new rest_api();
        CTRL_CONN = new controller_connection();
        MQTT = new _mqtt();

        CTRL_REGA = new controllerRega();

        WEATHER = new weather();
        MQTT.onWeatherInfoArrived = WEATHER.render;
 
        // init setup screen
        CONFIG = new setup(DB.config);

        // init cenas screen
        CENAS = new sensors();
    
        //TESTED
        window.onbeforeunload = MQTT.disconnect;

    }catch(err){
        log.error(err);
    }
};

window.addEventListener('load', function() {
    // some global vars to update upfront
    TEST = is_test_env();
    config_client_id(CLIENT_ID);

    MAIN = new main(); 

    CTRL_CONN.connected_event.registerObserver(DB, DB.connected_event);
    REST_API.call_server_sync(CTRL_CONN.url, CTRL_CONN.register_client_sync);
    CTRL_REGA.view.render_all();

    MAIN_VIEW.hide_loader();
});
