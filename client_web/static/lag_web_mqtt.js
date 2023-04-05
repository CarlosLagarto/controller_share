"use strict";

//TESTED
function ConnectionEvent(){
    this.observers = [];
    //registamos os observadores
    this.registerObserver = function(observer){
        if (this.observers.indexOf(observer) === -1){
            this.observers.push(observer)
        }
    }.bind(this);
    //removemos o registo do observador
    this.unregisterObserver = function(observer){
        let index = this.observers.indexOf(observer);
        if (index > -1){
            this.observers.splice(index, 1);
        }
    }.bind(this);

    this.notifyObservers = function(status){
        let observer = null, i = 0, l = this.observers.length;
        let ocopy = [];
        //esta cópia é porque com os callbacks e "threads" á aqui uma racing condition e pode-se chamar observadores que entre a 
        //construção da lista e a chamada foram unsubscribed
        for(i= 0; i < l; i += 1){
            ocopy.push(this.observers[i]);
        }
        //notificamos todos os observadores
        l = ocopy.length;
        for(i= 0; i < l; i += 1){
            try{
                observer = ocopy[i];
                if (this.observers.indexOf(observer) !== -1) {
                    observer.notifyConnection(status);
                }
            }catch(err){
                log.error(err);
            }            
        }
        ocopy = null;
        observer = null;
    }.bind(this);

    /*
    //todos os observadores tem que implementar esta função
    function Observer(){ this.notifyConnection = function(status){} ::::::: ;}
    */
}

//TESTED
function EndPoint(mqtt_obj) {
    this.mqtt_obj =  mqtt_obj;

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
        if (!self._cfg){ self._cfg = { host: "lagarto-lx.privatedns.org", client_id: CLIENT_ID, port: 9001}}
        if (!self._cfg.host){ self.host = "lagarto-lx.privatedns.org"; }
        if (!self._cfg.client_id){ self.client_id = CLIENT_ID; }
        if (!self._cfg.port){ self.port = 9001; }

        localStorage.setItem(BD_MQTT_TBL, JSON.stringify(self._cfg));
    };

    this.save = function(){
        localStorage.setItem(BD_MQTT_TBL, JSON.stringify(this._cfg));
        this.mqtt_obj._configureClientOptions(this);
    }
}

//TESTED
var _mqtt = function() {

    this.endPoint = new EndPoint(this);
    this.endPoint.load();

    this.connected_flag = false;
    this.isDisconnected = () => !this.connected_flag;
    // noinspection JSUnusedGlobalSymbols
    this.isConnected = () => this.connected_flag;

    this.client = null;

    //TESTED
    this.debug_callback = function (json){console.log(json)};

    this.SubscriptionsCallBacks = {};
    this.SubscriptionsCallBacks[MSG.T.STC_WEATHER ]          = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SYNC_DB]           = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_FULLDB]        = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT]         = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.SERVER_CONNECTION]     = this.debug_callback;    
    this.SubscriptionsCallBacks[MSG.T.STC_SND_ALERT_RESET]   = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_SND_LOG_ERROR]     = this.debug_callback;
    this.SubscriptionsCallBacks[MSG.T.STC_WEATHER_HIST]      = this.debug_callback;

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
    // noinspection JSUnusedGlobalSymbols
    this.subscriptions = [];

    this.haveController = false;

    // TESTED
    //inicialização do objeto cache
	this.msgs_cache = new CacheObject({key: BD_UUID_TBL, maxSize:MAX_CACHE_SIZE});
    this.msgs_cache.populate();

    this.connectionEvent = new ConnectionEvent();

    //TESTED
    this.notConnected = function (){
        this.haveController = false;
        this.connectionEvent.notifyObservers(false)
    }.bind(this);

    //TESTED
    this.waitMQTTTimeoutCallback = function (){
        let smsg = "A tentar religar ao broker...";
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        if (!this.connected_flag){
            this._connect()
        }
    }.bind(this);

    this.waitMQTTTimeout = new ThreadSimul("waitMQTTTimeout", WAIT_MQTT_TIMEOUT_INTERVAL, this.waitMQTTTimeoutCallback);

    //
    //client callbacks
    //
    //TESTED
    this.onConnectionLost = function (respObj){

        this.connected_flag = false;
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
        setTimeout(this.disconnect, 0);
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
                }else{  //só executamos se correu bem atrás
                    callback = self.SubscriptionsCallBacks[ topic ];
                    //# REVIEW e é aqui que vamos trabalhar a cache para ver se há coisas estranhas a acontecer, evitar repetir
                    //# comandos, temas de idempotência e etcs
                    process = (topic !== MSG.T.SERVER_CONNECTION); //só executamos os callbacks das mensagens != HB
                    if (topic === MSG.T.SERVER_CONNECTION){
                        if (message.body.status === CONNECTION.ONLINE){
                            self.haveController = true;
                        }else{
                            self.haveController = false;
                        }
                        setTimeout(self.connectionEvent.notifyObservers, 0, self.haveController);
                    }
                    else{
                        if (MSG.IdempotentMessages.includes(topic)){
                            uuid = message.uuid();
                            if (self.msgs_cache.have(uuid)) {
                                // mensagem já recebida - ignorar
                                msg = `Mensagem do server: ${message.client_id()} com o tópico: ${message.Topic()} enviada duas vezes.  Ignorada!`;
                                log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(msg, uuid));
                                process = false;
                            }
                            setTimeout(self.updateCache, 0, uuid, message);
                        }
                        //qualquer mensagem que recebemos do servidor é conceptualmente um HeartBeat.
                        // a mensagem especifica para o HeartBeat só é necessária porque em produção pode levar algum tempo
                        // até receber mensagens e nesse contexto o HeartBeat é necessário.  Em development em modo warp não é.
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
        } catch(err){
            log.error(err);
        } finally{
            message = null;
            callback = null;self = null;
        }
    }.bind(this);

    //TESTED
    this.updateCache = function(uuid, message){
        this.msgs_cache.addObj(uuid, message);
        this.msgs_cache.save();
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
        let smsg = `MQTT - Ligação falhou. Code: ${respObj.errorCode} message: ${respObj.errMessage}`;
        log.addEntry(LOG_ENTRY_TYPE.ERROR, log.newEntry(smsg));
        this.connected_flag = false;
        this.waitMQTTTimeout.start();
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
            message.body["status"] = CONNECTION.OFFLINE;
        }else{
            message.body["status"] = CONNECTION.ONLINE;
        }
        let lwt = new Paho.Message(message.get_json_str());
        lwt.destinationName = MSG.T.CLIENT_CONNECTION;
        lwt.qos = 0;
        lwt.retained = true;
        message = null;
        return lwt
    };

    //TESTED
    this._connect = function (){
        this.connect_options["willMessage"] = this.connectionMessage(false);
        // Connect the client, with a Last-Will-and-Testament
        this.client.connect(this.connect_options);
    };

    //TESTED
    this._configureClientOptions = function (endPoint){
        let self = this;
        self.client = new Paho.Client(endPoint.host, endPoint.port, endPoint.client_id);

        self.client.onConnectionLost = self.onConnectionLost;
        self.client.onMessageArrived = self.onMessageArrived;
        self.client.onConnected = self.onConnected;
        self = null;
    };

    //TESTED
    this.connect = function (_endPoint) {
        let self = this;
        if (self.connected_flag){
            self.disconnect();
            self.connected_flag = false;
        }
        if (!_endPoint){
            _endPoint = self.endPoint;
        }
        let smsg = `A ligar o cliente: ${_endPoint.client_id} ao host: ${_endPoint.host} na porta: ${_endPoint.port}`;
        log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(smsg));
        try{
            self._configureClientOptions(_endPoint);
            self._connect();
            }
        catch(error){
            smsg = `Erro a ligar ao broker MQTT. ${error.message}`;
            // error.message = smsg + error.message;
            log.error(smsg);
        }
        self = null;
    }.bind(this);
    
    //TESTED
    this._send_message = function (topic, msg, qos, retained){
        let message = null;
        if (!this.connected_flag){
            let out_msg = "Cliente MQTT não ligado.  Não vale a pena enviar mensagens.";
            log.addEntry(LOG_ENTRY_TYPE.LOG, log.newEntry(out_msg));
            return false;
        }
        try{
            message = new Paho.Message(msg);
            message.destinationName = topic;
            message.qos = qos;
            message.retained = retained;
            this.client.send(message);
        } catch(error){
            let smsg = `Erro a enviar a mensagem MQTT: ${msg} no tópico: ${topic}`;
            smsg += error.message;
            error.message = smsg + error.message;           
            log.error(error);             
        } finally{
            message = null;
        }
        return true;
    };

    //TESTED
    this.SendMessage = function (message){
        let smsg = message.get_json_str();
        this._send_message(message.Topic(), smsg, QoS, false);
    };

    this.connect_options = {
        timeout: 5,
        keepAliveInterval: 60,
        cleanSession: true,
        useSSL: false,
        onSuccess: this.onSuccess,
        onFailure: this.onFailure,
        reconnect: true,
        mqttVersion: 3,
    };

    //TESTED
    this.getSubscribeOptions = function (topic) {
        return {qos: QoS,
                onSuccess: this.onSubscribeSuccess,
                invocationContext: {topic: topic},
                onFailure: this.onSubscribeFailure}
    };

};

