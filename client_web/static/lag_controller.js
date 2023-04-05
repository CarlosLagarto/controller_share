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
var ctrlRega = null;
var MAIN = null;
var MAIN_VIEW = null;

var main = function(){
    //variáveis globais
    this.controllerConnection = document.getElementById("controllerConnection");
    this.infoDate = document.getElementById("infoDate");

    this.updateDate = function(date){
        infoDate.innerHTML = date;        
    };
    this.connected = function(status, date){
        this.updateDate(date)
        addClass(controllerConnection.classList, "connected");
    };

    this.disconnected = function (){
        removeClass(controllerConnection.classList, "connected");
    };

    this.notifyConnection = function(status){
        if (status){
            let d = moment().format();
            this.connected(status, d);
        } else {
            this.disconnected();
        }
    }.bind(this);

    try{

        moment.defaultFormat = DATE_LONG_FORMAT;

        MAIN_VIEW = new main_view();
        MAIN_VIEW.ini_application_ui();

        MAIN_VIEW.StatusMsg.setTemp("A ligar ao broker e a obter info.", 5);

        log = new logs(); 
        log.populate();
        log.render(null, true);
        window.addEventListener('error', log.error);

        MSG = new _MSG();
        MQTT = new _mqtt();

        //TESTED
        window.onbeforeunload = MQTT.disconnect;

        MQTT.connectionEvent.registerObserver(this);
        MQTT.connect();

        DB = new db();
        DB.open();

        MQTT.onSTCLogErrorArrived = log.STCLogArrived;
        
        ctrlRega = new controllerRega();

        WEATHER = new weather();
        MQTT.onWeatherInfoArrived = WEATHER.render;

        //inicializa ecran de setup
        CONFIG = new setup(DB.config);
        CONFIG.render();

    }catch(err){
        log.error(err);
    }
};


window.addEventListener('load', function() {
    MAIN = new main(); 
});
