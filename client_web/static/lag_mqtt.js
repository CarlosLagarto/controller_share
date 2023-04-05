"use strict";

//TESTED
function EndPoint() {
    this._cfg = JSON.parse(localStorage.getItem(BD_MQTT_TBL));

    Object.defineProperty(this, "host", {
        get()  { return this._cfg.host;},
        set(x) { this._cfg.host = x; }
    });
    Object.defineProperty(this, "client_id", {
        get()  { return this._cfg.client_id;},
        set(x) { this._cfg.client_id = x; }
    });
    Object.defineProperty(this, "port", {
        get()  { return this._cfg.port;},
        set(x) { this._cfg.port = x; }
    });

    this.load = function(){
        let self = this;
        if (!self._cfg){ self._cfg = { host: HOST, client_id: CLIENT_ID, port: 443}}
        if (!self._cfg.host){ self._cfg.host = HOST; }
        if (self._cfg.client_id != CLIENT_ID ){ self._cfg.client_id = CLIENT_ID; }
        if (!self._cfg.port){ self._cfg.port = 443; }

        localStorage.setItem(BD_MQTT_TBL, JSON.stringify(self._cfg));
    };

    this.save = function(){
        localStorage.setItem(BD_MQTT_TBL, JSON.stringify(this._cfg));
    }
    this.load();
}

//TESTED
var _mqtt = function() {    
    //TESTED
    this._configureClientOptions = function (){
        let self = this;
        self.client = new Paho.Client(self.endPoint.host, self.endPoint.port, self.endPoint.client_id);

        self.client.onConnectionLost = self.onConnectionLost;
        self.client.onMessageArrived = self.onMessageArrived;
        self.client.onConnected = self.onConnected;
        self = null;
    };
    
    this.isDisconnected = () => !this.connected_flag;
    this.isConnected = () => this.connected_flag;

    //TESTED
    this.debug_callback = function (json){console.log(json)};

    Object.defineProperty(this, "onWeatherInfoArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_WEATHER] = x; }
    });
    Object.defineProperty(this, "onSTCSyncDBArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_SYNC_DB] = x; }
    });
    Object.defineProperty(this, "onSTCAlertArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT] = x; }
    });
    Object.defineProperty(this, "onSTCAlertResetArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT_RESET] = x; }
    });
    Object.defineProperty(this, "onSTCLogErrorArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_SND_LOG_ERROR] = x; }
    });
    Object.defineProperty(this, "onSTCWeatherHistArrived", {
        set(x) { this.SubscriptionsCallBacks[MSG.T.STC_WEATHER_HIST] = x; }
    });

    //TESTED
    this.notConnected = function (){
        setTimeout(this.disconnected_event.notifyObservers, 0, null);
    }.bind(this);

    //TESTED
    this.waitMQTTTimeoutCallback = function (){
        let self = this;
        let smsg = "A tentar religar ao broker...";
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        if (!self.connected_flag){
            self._connect()
        }
        self = null;
    }.bind(this);

    //
    //client callbacks
    //
    //TESTED
    this.onConnectionLost = function (respObj){
        let self = this;
        self.connected_flag = false;
        if(respObj.errorCode === 0){
            let smsg = `MQTT desligado.  Código: ${respObj.errorCode}`;
            log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        }
        else{
            let smsg = `MQTT desligado.  Código: ${respObj.errorCode}: ${MQTT_ERROR[respObj.errorCode]}`;
            log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(smsg));
            let s1 = respObj.errorMessage;
            if (!s1){
                smsg = `Mensagem do MQTT: ${s1}`;
                log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(smsg));
            }
        }
        setTimeout(self.disconnect, 0);
    }.bind(this);

    //TESTED
    this.onMessageArrived = function (r_message){
        let str_message = r_message.payloadString;
        let msg = `Mensagem MQTT recebida: ${str_message} com o Tópico: ${r_message.destinationName}`;
        let message = null;
        let callback = null;
        let uuid = "";
        let process = true;
        let self = this;
        try{
            let topic = r_message.destinationName;
            if (self.SubscriptionsCallBacks.hasOwnProperty(topic)){
                message = MSG.BuildMessage(topic, str_message);
                if (!message) {
                    msg = `Erro na mensagem recebida:${msg}`;
                    uuid = str_message.slice(str_message.indexOf("uuid:") + 5, 21);
                    if (uuid.length === 21){
                        log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(msg, uuid));
                    }else{
                        log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(msg));
                    }
                }else{  
                    // adjust time delta with each server msg 
                    DELTA_TIME_MS = (new Date()).getTime() - (message.msg.header.time * 1000);
                    callback = self.SubscriptionsCallBacks[ topic ];
                    process = (topic !== MSG.T.SERVER_CONNECTION); //só executamos os callbacks das mensagens != HB
                    if (topic === MSG.T.SERVER_CONNECTION){
                        if (message.msg.status === CONNECTION.ONLINE){
                            CTRL_CONN.server_connected();
                        }else{
                            CTRL_CONN.server_disconnected();
                        }
                    }
                    else{
                        if (MSG.IdempotentMessages.includes(topic)){
                            uuid = message.uuid();
                            if (self.msgs_cache.have(uuid)) {
                                // duplicated msg - ignore
                                msg = `Mensagem do server: ${message.client_id()} com o tópico: ${topic} enviada duas vezes.  Ignorada!`;
                                log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(msg, uuid));
                                process = false;
                            }
                            setTimeout(self.updateCache, 0, uuid, message);
                        }
                        // every server msg is conceptually a heart beat
                        // the herat beat specific msg is nevertheless needed because it can span some time without msgs
                        if (process ) {
                            if(typeof(callback) !== "undefined"){
                                setTimeout(callback, 0, message);
                            }else{
                                let s1 = `callback das mensagens não definido.  Tópico:${topic}`;
                                log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(s1));
                            }
                        }
                    }
                }
            }
            // else, unknown topic - ignore
        } catch(err){
            log.error(err);
        } finally{
            message = null;
            callback = null;
            self = null;
        }
    }.bind(this);

    //TESTED
    this.updateCache = function(uuid, message){
        let self = this;
        self.msgs_cache.addObj(uuid, message);
        self.msgs_cache.save();
        self = null;
    }.bind(this);

    //TESTED
    this.onConnected = function (reconnect, uri){
        this.connected_flag = true;
        this.waitMQTTTimeout.stop();

        if (reconnect === true){
            let smsg = "Broker caiu.  Religámos e vamos resubscrever";
            log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        }
        // smsg = "uri: " + uri;
        setTimeout(this._subscribe, 0);
    }.bind(this);

    //TESTED
    this._subscribe = function(){
        let self = this;
        try{
            // Once a connection has been made, make a subscription and send a the online message.
            for (let subscription in self.SubscriptionsCallBacks) {
                self.client.subscribe(subscription, self.getSubscribeOptions(subscription));
            }
            self.connectionMessage(true);
            setTimeout(self.connected_event.notifyObservers, 0, null);            
        }
        catch(error){
            log.error(error);
        }        
    }.bind(this);

    //
    //connected options callbacks
    //
    //TESTED
    this.onFailure = function (respObj ) {
        let self = this;
        let smsg = `MQTT - Ligação falhou. Code: ${respObj.errorCode} message: ${respObj.errMessage}`;
        log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(smsg));
        self.connected_flag = false;
        setTimeout(self.disconnected_event.notifyObservers(),0, null);
        self.waitMQTTTimeout.start();
        self = null;
    }.bind(this);

    //TESTED
    this.onSuccess = function (invocationContext) {
        this.connected_flag = true;
        let smsg = `Cliente MQTT: ${this.endPoint.client_id} ligado a: ${this.endPoint.host} na porta: ${this.endPoint.port}`;
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
    }.bind(this);

    //TESTED
    this.onSubscribeSuccess = function (respObj){
        let smsg = `Subscrição MQTT OK. QoS: ${respObj.grantedQos} no tópico ${respObj.invocationContext.topic}`;
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
    }.bind(this);

    //NOTTESTED
    this.onSubscribeFailure = function (invocationContext, errorCode, errorMessage){
        let smsg = `A Subscrição MQTT falhou. Código: ${errorCode} para o tópico: ${invocationContext.topic}`;
        log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(smsg));
    }.bind(this);

    //
    // actions functions
    //
    //TESTED
    this.disconnect = function (){
        let smsg = "A desligar o cliente MQTT.";
        let self = this;
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        if (self.connected_flag){
            try{
                if (self.client && self.client.isConnected()) {
                    self.connectionMessage(false);
                    self.client.disconnect();
                }
            }
            catch(error){
                smsg = "Erro a desligar o cliente MQTT." + error.message;
                error.message = smsg + error.message;
                log.error(smsg)
            }
            self.notConnected();
            self.waitMQTTTimeout.stop();
        }
        self = null;
    }.bind(this);

    // TESTED
    this.connectionMessage = function (value){
        let message = MSG.BuildMessage(MSG.T.CLIENT_CONNECTION);
        if (value){
            message.msg["status"] = CONNECTION.OFFLINE;
        }else{
            message.msg["status"] = CONNECTION.ONLINE;
        }
        let lwt = new Paho.Message(message.get_json_str());
        lwt.destinationName = MSG.T.CLIENT_CONNECTION;
        lwt.qos = 0;
        lwt.retained = true;
        message = null;
        return lwt
    }.bind(this);

    //TESTED
    // connect action
    this._connect = function (){
        let self = this;
        self.connect_options["willMessage"] = self.connectionMessage(false);
        // Connect the client, with a Last-Will-and-Testament
        self.client.connect(self.connect_options);
        self = null;
    }.bind(this);

    //TESTED
    // Lógica de controlo do connect
    this.connect = function () {
        let self = this;
        if (self.connected_flag){
            self.disconnect();
            self.connected_flag = false;
        }
        let smsg = `A ligar o cliente: ${self.endPoint.client_id} ao host: ${self.endPoint.host} na porta: ${self.endPoint.port}`;
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        try{
            self._configureClientOptions();
            self._connect();
            }
        catch(error){
            smsg = `Erro a ligar ao broker MQTT. ${error.message}`;
            log.error(smsg);
        }
        self = null;
    }.bind(this);
    
    //TESTED
    this._send_message = function (topic, msg, qos, retained){
        let message = null;
        let self = this;
        if (!self.connected_flag){
            let out_msg = "Cliente MQTT não ligado.  Não vale a pena enviar mensagens.";
            log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(out_msg));
            return false;
        }
        try{
            message = new Paho.Message(msg);
            message.destinationName = topic;
            message.qos = qos;
            message.retained = retained;
            self.client.send(message);
        } catch(error){
            let smsg = `Erro a enviar a mensagem MQTT: ${msg} no tópico: ${topic}`;
            smsg += error.message;
            error.message = smsg + error.message;           
            log.error(error);             
        } finally{
            message = null;
            self = null;
        }
        return true;
    }.bind(this);

    //TESTED
    this.SendMessage = function (message){
        let smsg = message.get_json_str();
        this._send_message(message.topic(), smsg, QoS, false);
    }.bind(this);

    //TESTED
    this.getSubscribeOptions = function (topic) {
        return {qos: QoS,
                onSuccess: this.onSubscribeSuccess,
                invocationContext: {topic: topic},
                onFailure: this.onSubscribeFailure}
    }.bind(this);

    this.start = function(){
        console.log(CLIENT_ID);
        let self = this;

        self.endPoint = new EndPoint();

        if (!self.initialized){
            self.connected_event.registerObserver(CTRL_CONN, CTRL_CONN._connected_event);
            self.disconnected_event.registerObserver(CTRL_CONN, CTRL_CONN._disconnected_event);
            self.onSTCSyncDBArrived = DB.syncFromServer;
            self.onSTCLogErrorArrived = log.STCLogArrived;
            self.initialized = true;
        }
        if (!self.connected_flag){
            self.connect();
        }

        CONFIG.render();
        self = null;
    }.bind(this);

    this.initialized = false;
    this.endPoint = new EndPoint();
    this.connected_flag = false;
    this.client = null;
    this.subscriptions = [];

    this.SubscriptionsCallBacks = {};
    this.SubscriptionsCallBacks[MSG.T.STC_WEATHER ]          = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SYNC_DB]           = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_FULLDB]        = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT]         = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.SERVER_CONNECTION]     = this.debug_callback;    
    this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT_RESET]   = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_LOG_ERROR]     = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_WEATHER_HIST]      = this.debug_callback;

    // TODO - handle CENAS topics

    // TESTED
    // cache init
	this.msgs_cache = new CacheObject({key: BD_UUID_TBL, maxSize:MAX_CACHE_SIZE});
    this.msgs_cache.populate();

    this.connected_event = new LagEvent();
    this.disconnected_event = new LagEvent();
        
    this.waitMQTTTimeout = new ThreadSimul("waitMQTTTimeout", WAIT_MQTT_TIMEOUT_INTERVAL, this.waitMQTTTimeoutCallback);

    this.connect_options = {
        // timeout: 5,
        keepAliveInterval: 120,
        cleanSession: true,
        // MQTT broker do not use SSL.  It's open behind the proxy
        // But we need this because the browser upgrades https to web sockets, and uses port 443, so ssl have to be true, otherwise the browser complains
        useSSL: true,
        onSuccess: this.onSuccess,
        onFailure: this.onFailure,
        reconnect: true,
        mqttVersion: 3,
    };
};

