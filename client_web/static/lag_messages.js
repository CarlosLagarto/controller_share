"use strict";

var _MSG = function () {

    // TESTED
    //# organize concepts
    if (!TEST) {
        this.T = {
            STC_WEATHER: "LAGARTO_CONTROLLER/STC/DATA/WEATHER",
            STC_SYNC_DB: "LAGARTO_CONTROLLER/STC/DATA/SYNCDB",
            STC_SND_FULLDB: "LAGARTO_CONTROLLER/STC/DATA/FULLDB",
            STC_SND_ALERT: "LAGARTO_CONTROLLER/STC/DATA/ALERT",
            STC_SND_ALERT_RESET: "LAGARTO_CONTROLLER/STC/DATA/ALERT_RESET",
            STC_SND_LOG_ERROR: "LAGARTO_CONTROLLER/STC/DATA/LOG_ERROR",
            STC_WEATHER_HIST: "LAGARTO_CONTROLLER/STC/DATA/WEATHER/HIST/PUT",

            CTS_STOP_CYCLE: "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE",
            CTS_STOP_SECTOR: "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR",
            CTS_STATUS_CHANGE_MODE: "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE",
            CTS_STATUS_SHUTDOWN: "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown",
            CTS_FORCE_CYCLE: "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE",
            CTS_FORCE_SECTOR: "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR",
            CTS_GET_FULLDB: "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB",
            CTS_SYNC_DB: "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB",
            CTS_GET_WEATHER_HIST: "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET",

            CLIENT_CONNECTION: "LAGARTO_CONTROLLER/CONNECTION/CLIENT",
            SERVER_CONNECTION: "LAGARTO_CONTROLLER/CONNECTION/SERVER"
            //        DEVICE_1_CONNECTION     : "LAGARTO_CONTROLLER/CONNECTION/DEVICE1"
        };
    } else {
        this.T = {
            STC_WEATHER: "LAGARTO_CONTROLLER/STC/DATA/WEATHER/TEST",
            STC_SYNC_DB: "LAGARTO_CONTROLLER/STC/DATA/SYNCDB/TEST",
            STC_SND_FULLDB: "LAGARTO_CONTROLLER/STC/DATA/FULLDB/TEST",
            STC_SND_ALERT: "LAGARTO_CONTROLLER/STC/DATA/ALERT/TEST",
            STC_SND_ALERT_RESET: "LAGARTO_CONTROLLER/STC/DATA/ALERT_RESET/TEST",
            STC_SND_LOG_ERROR: "LAGARTO_CONTROLLER/STC/DATA/LOG_ERROR/TEST",
            STC_WEATHER_HIST: "LAGARTO_CONTROLLER/STC/DATA/WEATHER/HIST/PUT/TEST",

            CTS_STOP_CYCLE: "LAGARTO_CONTROLLER/CTS/STATUS/STOP_CYCLE/TEST",
            CTS_STOP_SECTOR: "LAGARTO_CONTROLLER/CTS/STATUS/STOP_SECTOR/TEST",
            CTS_STATUS_CHANGE_MODE: "LAGARTO_CONTROLLER/CTS/STATUS/CHANGE_MODE/TEST",
            CTS_STATUS_SHUTDOWN: "LAGARTO_CONTROLLER/CTS/STATUS/ShutDown/TEST",
            CTS_FORCE_CYCLE: "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_CYCLE/TEST",
            CTS_FORCE_SECTOR: "LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST",
            CTS_GET_FULLDB: "LAGARTO_CONTROLLER/CTS/DATA/GET_FULLDB/TEST",
            CTS_SYNC_DB: "LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST",
            CTS_GET_WEATHER_HIST: "LAGARTO_CONTROLLER/CTS/DATA/WEATHER/HIST/GET/TEST",

            CLIENT_CONNECTION: "LAGARTO_CONTROLLER/CONNECTION/CLIENT/TEST",
            SERVER_CONNECTION: "LAGARTO_CONTROLLER/CONNECTION/SERVER/TEST"
            //        DEVICE_1_CONNECTION     : "LAGARTO_CONTROLLER/CONNECTION/DEVICE1/TEST"
        };
    }

    const TOPIC_IN = {};
    TOPIC_IN[this.T.STC_WEATHER] = this.T.STC_WEATHER;
    TOPIC_IN[this.T.STC_SYNC_DB] = this.T.STC_SYNC_DB;
    TOPIC_IN[this.T.STC_SND_FULLDB] = this.T.STC_SND_FULLDB;
    TOPIC_IN[this.T.SERVER_CONNECTION] = this.T.SERVER_CONNECTION;
    TOPIC_IN[this.T.STC_SND_ALERT] = this.T.STC_SND_ALERT;
    TOPIC_IN[this.T.STC_SND_ALERT_RESET] = this.T.STC_SND_ALERT_RESET;
    TOPIC_IN[this.T.STC_SND_LOG_ERROR] = this.T.STC_SND_LOG_ERROR;
    TOPIC_IN[this.T.STC_WEATHER_HIST] = this.T.STC_WEATHER_HIST;

    const TOPIC_OUT = {};
    TOPIC_OUT[this.T.CTS_GET_FULLDB] = this.T.CTS_GET_FULLDB;
    TOPIC_OUT[this.T.CTS_SYNC_DB] = this.T.CTS_SYNC_DB;
    TOPIC_OUT[this.T.CLIENT_CONNECTION] = this.T.CLIENT_CONNECTION;
    TOPIC_OUT[this.T.CTS_STOP_CYCLE] = this.T.CTS_STOP_CYCLE;
    TOPIC_OUT[this.T.CTS_STOP_SECTOR] = this.T.CTS_STOP_SECTOR;
    TOPIC_OUT[this.T.CTS_STATUS_CHANGE_MODE] = this.T.CTS_STATUS_CHANGE_MODE;
    TOPIC_OUT[this.T.CTS_FORCE_CYCLE] = this.T.CTS_FORCE_CYCLE
    TOPIC_OUT[this.T.CTS_FORCE_SECTOR] = this.T.CTS_FORCE_SECTOR;
    TOPIC_OUT[this.T.CTS_GET_WEATHER_HIST] = this.T.CTS_GET_WEATHER_HIST;

    const TOPIC_BASIC_OUT = {};
    TOPIC_BASIC_OUT[this.T.CTS_STATUS_SHUTDOWN] = this.T.CTS_STATUS_SHUTDOWN;

    this.IdempotentMessages = [ this.T.STC_SYNC_DB, this.T.STC_SND_FULLDB, this.T.CTS_SYNC_DB, this.T.STC_SND_ALERT, this.T.STC_SND_LOG_ERROR];

    this.inMessages = [this.T.STC_WEATHER, this.T.STC_SYNC_DB, this.T.STC_SND_FULLDB, this.T.SERVER_CONNECTION, this.T.STC_SND_ALERT,
                        this.T.STC_SND_ALERT_RESET, this.T.STC_SND_LOG_ERROR, this.T.STC_WEATHER_HIST];

    this.outMessages = [this.T.CTS_GET_FULLDB, this.T.CTS_SYNC_DB, this.T.CLIENT_CONNECTION, this.T.CTS_STOP_CYCLE, this.T.CTS_STOP_SECTOR,
                        this.T.CTS_STATUS_CHANGE_MODE, this.T.CTS_FORCE_CYCLE, this.T.CTS_FORCE_SECTOR, this.T.CTS_GET_WEATHER_HIST,
                        this.T.CTS_STATUS_SHUTDOWN];

    this.withMessageInfo = [this.T.STC_WEATHER, this.T.STC_SYNC_DB, this.T.STC_SND_FULLDB, this.T.CTS_GET_FULLDB, this.T.CTS_SYNC_DB,
                            this.T.CTS_STOP_CYCLE, this.T.CTS_STOP_SECTOR, this.T.CTS_FORCE_CYCLE, this.T.CTS_FORCE_SECTOR,
                            this.T.CTS_STATUS_CHANGE_MODE, this.T.STC_SND_ALERT, this.T.STC_SND_LOG_ERROR, this.T.STC_WEATHER_HIST];


    const TOPIC_OUT_TYPE = {};
    TOPIC_OUT_TYPE[this.T.CTS_GET_FULLDB] = "GetFullDB";
    TOPIC_OUT_TYPE[this.T.CTS_SYNC_DB] = "DBSync";
    TOPIC_OUT_TYPE[this.T.CLIENT_CONNECTION] = "Connection";
    TOPIC_OUT_TYPE[this.T.CTS_STOP_CYCLE] = "Cycle";
    TOPIC_OUT_TYPE[this.T.CTS_STOP_SECTOR] = "Sector";
    TOPIC_OUT_TYPE[this.T.CTS_STATUS_CHANGE_MODE] = "ChangeMode";
    TOPIC_OUT_TYPE[this.T.CTS_FORCE_CYCLE] = "Cycle";
    TOPIC_OUT_TYPE[this.T.CTS_FORCE_SECTOR] = "Sector";
    TOPIC_OUT_TYPE[this.T.CTS_GET_WEATHER_HIST] = "WeatherHistory";
    TOPIC_OUT_TYPE[this.T.CTS_STATUS_SHUTDOWN] = "ShutDown";

    this.TOPIC = {};
    addMethods(this.TOPIC, TOPIC_IN);
    addMethods(this.TOPIC, TOPIC_OUT);
    addMethods(this.TOPIC, TOPIC_BASIC_OUT);

    // TESTED
    // common basic atributes
    this.basic_attributes = function () {
        return {
            msg: {
                "type": "null",
                header: {
                    topic: "",
                    client_id: "",
                    time: 0,
                    uuid: "" 
                },
                Topic: function () { return this.header.topic; },

            },
            set_json: function (json_str) { this.msg = JSON.parse(json_str); },
            get_json: function () { return this.msg; },
            get_json_str: function () { return JSON.stringify(this.msg); },

            client_id: function () { return this.msg.header.client_id; },
            topic: function () { return this.msg.header.topic; },
            uuid: function () { return this.msg.header.uuid; }
        }
    }.bind(this);

    // TESTED
    this.BuildMessage = function (topic, message) {
        let object = {};
        let _in = false;
        let obj_in = null;
        let self = this;
        try {
            addMethods(object, new self.basic_attributes());

            if (self.inMessages.includes(topic)) {
                obj_in = JSON.parse(message); 
                _in = true;
            } else {// out msg
                object.msg.header.client_id = CLIENT_ID;
                object.msg.header.time = get_unix_now_time();
                object.msg.type = TOPIC_OUT_TYPE[topic];
            }

            object.msg.header.topic = topic;

            if (_in) {
                addMethods(object.msg, obj_in);
            } else {// out msg
                object.msg.header.uuid = new UUID(1).format(); 
                if (self.withMessageInfo.includes(topic)) { 
                    addMethods(object.msg, message);
                }
            }
        } catch (err) {
            log.error(err);
            object = null;
        }
        obj_in = null;
        return object;
    }.bind(this);
};
